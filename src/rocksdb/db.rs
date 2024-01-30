#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::graph::{Edge, Node};
use prost::Message; // need the trait to encode protobuf
use rocksdb::{DBWithThreadMode, Direction, IteratorMode, Options, SingleThreaded, DB};
use std::convert::TryInto;
use std::error;
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone)]
struct ErrBadDbPath {
    symlink: bool,
    path: String,
}

impl ErrBadDbPath {
    fn file(path: &str) -> ErrBadDbPath {
        ErrBadDbPath {
            symlink: false,
            path: path.to_string(),
        }
    }
    fn symlink(path: &str) -> ErrBadDbPath {
        ErrBadDbPath {
            symlink: true,
            path: path.to_string(),
        }
    }
}

impl fmt::Display for ErrBadDbPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad path {:?}, symlink = {:?}", self.path, self.symlink)
    }
}

impl error::Error for ErrBadDbPath {}

fn check_path(path: &str) -> Result<&Path, Box<dyn error::Error>> {
    let p = Path::new(path);
    match p.try_exists() {
        Err(e) => return Err(Box::new(e)),
        Ok(false) => return Ok(p),
        Ok(true) => {
            if p.is_file() {
                error!("Path is a file: {}", path);
                return Err(Box::new(ErrBadDbPath::file(path)));
            }
            if p.is_symlink() {
                error!("Path is a file: {}", path);
                return Err(Box::new(ErrBadDbPath::symlink(path)));
            }
            Ok(p)
        }
    }
}

#[test]
fn test_check_path() {
    assert_eq!(check_path("/bin").unwrap(), Path::new("/bin"));

    // expect error -- this is a path to a file or symlink
    check_path("/bin/bash").unwrap_err();

    // Non-existent file path is ok.
    check_path("/i/dont/exist").unwrap();
}

pub trait DbInfo {
    fn path(&self) -> &str;
}

pub trait VisitKV {
    fn visit(&self, _: &[u8], _: &[u8]);
}

static CF_SYSTEM: &str = "cf.system";
static CF_DEFAULT: &str = "cf.default";
static CF_NODES: &str = "cf.nodes";
static CF_EDGES: &str = "cf.edges";
static SEQ_KEY: &str = "sequence";

pub fn init(info: &dyn DbInfo) -> Result<(), Box<dyn error::Error>> {
    trace!("Init path={}", info.path());
    let p = check_path(info.path())?;

    trace!("Db init at path = {}", info.path());

    let mut options = Options::default();
    options.set_error_if_exists(false);
    options.create_if_missing(true);
    options.create_missing_column_families(true);

    let cfs = DB::list_cf(&options, p).unwrap_or(vec![]);

    // open a DB with specifying ColumnFamilies
    let mut instance = DB::open_cf(&options, check_path(info.path())?, cfs.clone())?;

    for c in vec![CF_SYSTEM, CF_DEFAULT, CF_NODES, CF_EDGES].into_iter() {
        if cfs.iter().find(|cf| cf == &c).is_none() {
            // create a new ColumnFamily
            info!("Creating column family {:?}", c);
            let options = Options::default();
            let create_cf = instance.create_cf(c, &options)?;
            info!("Creating column family {:?}, result = {:?}", c, create_cf);
        } else {
            info!("Found column family {:?}", c);
        }
    }

    Ok(())
}

// TODO - Optimize this a bit more so that opening the database simply
// opens all the column families, without creating them (do that in "init").
fn open_db(info: &dyn DbInfo) -> Result<DBWithThreadMode<SingleThreaded>, Box<dyn error::Error>> {
    let mut options = Options::default();
    options.set_error_if_exists(false);
    options.create_if_missing(true);
    options.create_missing_column_families(true);

    // list existing ColumnFamilies in the given path. returns Err when no DB exists.
    let cfs = DB::list_cf(&options, check_path(info.path())?).unwrap_or(vec![]);
    // open a DB with specifying ColumnFamilies
    let mut instance = DB::open_cf(&options, check_path(info.path())?, cfs.clone())?;

    for c in vec![CF_SYSTEM, CF_DEFAULT, CF_NODES, CF_EDGES].into_iter() {
        if cfs.iter().find(|cf| cf == &c).is_none() {
            // create a new ColumnFamily
            let options = rocksdb::Options::default();
            let create_cf = instance.create_cf(c, &options).unwrap();
            warn!("Creating column family {:?}, result = {:?}", c, create_cf);
        }
    }

    Ok(instance)
}

fn next_id(db: &DBWithThreadMode<SingleThreaded>) -> Result<u64, Box<dyn error::Error>> {
    trace!("DB = {:?}", db);

    let id: u64;

    // TODO inefficient - refactor later
    let cf = db.cf_handle(CF_SYSTEM).unwrap();
    match db.get_cf(cf, SEQ_KEY.as_bytes()) {
        Ok(Some(v)) => {
            let be = v.try_into().unwrap_or_else(|v: Vec<u8>| {
                panic!("Expected a Vec of length {} but it was {}", 8, v.len())
            });

            id = u64::from_le_bytes(be) + 1;
            trace!("id read: {}", id);
        }
        Ok(None) => {
            id = 1;
        }
        Err(e) => {
            error!("error::Error retrieving value for {}: {}", SEQ_KEY, e);
            return Err(Box::new(e));
        }
    }

    // update the id before returning the value
    match db.put_cf(cf, SEQ_KEY.as_bytes(), id.to_le_bytes().to_vec()) {
        Ok(()) => Ok(id),
        Err(e) => {
            error!("Error updating sequence {}", SEQ_KEY);
            Err(Box::new(e))
        }
    }
}

pub fn put_node<'a>(
    info: &'a dyn DbInfo,
    node: &'a mut Node,
) -> Result<&'a Node, Box<dyn error::Error>> {
    trace!("node: {:?}", node);

    let db = open_db(info)?;
    let id = next_id(&db)?;
    node.id = id;

    info!("node: {:?}, encoded = {:?}", node, node.encode_to_vec());

    let cf = db.cf_handle(CF_NODES).unwrap();
    db.put_cf(cf, u64::to_le_bytes(id), node.encode_to_vec())?;

    Ok(node)
}

pub fn get_node(info: &dyn DbInfo, id: u64) -> Result<Option<Node>, Box<dyn error::Error>> {
    trace!("db: {:?}, node id: {:?}", info.path(), id);

    let db = open_db(info)?;

    let cf = db.cf_handle(CF_NODES).unwrap();
    match db.get_cf(cf, u64::to_le_bytes(id)) {
        Ok(Some(bytes)) => {
            trace!("Found node with id = {:?} found {:?}", id, bytes);
            let decoded: Node = Message::decode(&bytes[..])?;
            Ok(Some(decoded))
        }
        Ok(None) => {
            trace!("No node with id = {:?} found", id);
            Ok(None)
        }
        Err(e) => {
            error!("Error: {:?}", e);
            Err(Box::new(e))
        }
    }
}

pub fn put_edge<'a>(
    info: &'a dyn DbInfo,
    edge: &'a mut Edge,
) -> Result<&'a Edge, Box<dyn error::Error>> {
    trace!("edge: {:?}", edge);

    let db = open_db(info)?;
    let id = next_id(&db)?;
    edge.id = id;

    info!("edge: {:?}, encoded = {:?}", edge, edge.encode_to_vec());

    let cf = db.cf_handle(CF_EDGES).unwrap();
    db.put_cf(cf, u64::to_le_bytes(id), edge.encode_to_vec())?;

    Ok(edge)
}

#[allow(dead_code)]
pub fn get_edge(info: &dyn DbInfo, id: u64) -> Result<Option<Edge>, Box<dyn error::Error>> {
    trace!("db: {:?}, edge id: {:?}", info.path(), id);

    let db = open_db(info)?;

    let cf = db.cf_handle(CF_EDGES).unwrap();
    match db.get_cf(cf, u64::to_le_bytes(id)) {
        Ok(Some(bytes)) => {
            trace!("Found edge with id = {:?} found {:?}", id, bytes);
            let decoded: Edge = Message::decode(&bytes[..])?;
            Ok(Some(decoded))
        }
        Ok(None) => {
            trace!("No edge with id = {:?} found", id);
            Ok(None)
        }
        Err(e) => {
            error!("Error: {:?}", e);
            Err(Box::new(e))
        }
    }
}

pub fn put(info: &dyn DbInfo, key: &str, value: &str) -> Result<(), Box<dyn error::Error>> {
    trace!("Put path={}, key={}, value={}", info.path(), key, value);

    let db = open_db(info)?;
    trace!("DB = {:?}", db);

    match db.put(key.as_bytes(), value.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("Error retrieving value for {}: {}", key, e);
            Err(Box::new(e))
        }
    }
}

pub fn get(
    info: &dyn DbInfo,
    key: &str,
    visitor: &dyn VisitKV,
) -> Result<Option<String>, Box<dyn error::Error>> {
    trace!("Get path={}, key={}", info.path(), key);

    let db = open_db(info)?;
    trace!("DB = {:?}", db);

    match db.get(key.as_bytes()) {
        Ok(Some(v)) => {
            let result = String::from_utf8(v).unwrap();
            trace!("Finding '{}' returns '{}'", key, result);

            visitor.visit(key.as_bytes(), result.as_bytes());

            Ok(Some(result))
        }
        Ok(None) => {
            trace!("Finding '{}' returns None", key);
            Ok(None)
        }
        Err(e) => {
            error!("error::Error retrieving value for {}: {}", key, e);
            Err(Box::new(e))
        }
    }
}

pub fn delete(info: &dyn DbInfo, key: &str) -> Result<(), Box<dyn error::Error>> {
    trace!("Delete path={}, key={}", info.path(), key);
    let db = open_db(info)?;
    trace!("DB = {:?}", db);
    match db.delete(key.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("Error retrieving value for {}: {}", key, e);
            return Err(Box::new(e));
        }
    }
}

pub fn list(
    info: &dyn DbInfo,
    key: &str,
    visitor: &dyn VisitKV,
) -> Result<(), Box<dyn error::Error>> {
    trace!("List path={}, key={}", info.path(), key);
    let db = open_db(info)?;
    trace!("DB = {:?}", db);
    let iter = db.iterator(IteratorMode::From(key.as_bytes(), Direction::Forward));
    for item in iter {
        let (k, v) = item.unwrap();
        visitor.visit(&k, &v);
    }
    Ok(())
}
