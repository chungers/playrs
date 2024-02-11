#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::edge;
use crate::rocksdb::graph::{Edge, Node};
use crate::rocksdb::index::{Index, Indexes};
use crate::rocksdb::kv;
use crate::rocksdb::node;
use crate::rocksdb::All;

use prost::Message; // need the trait to encode protobuf
use rocksdb::{
    DBWithThreadMode, Direction, IteratorMode, Options, SingleThreaded, WriteBatchWithTransaction,
    DB,
};
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::path::Path;

pub trait DbInfo {
    fn path(&self) -> &str;
    fn options(&self) -> rocksdb::Options;
}

pub type Database = DBWithThreadMode<SingleThreaded>;
pub type Transaction = WriteBatchWithTransaction<false>;

pub trait VisitKV {
    fn visit(&self, _: &[u8], _: &[u8]);
}

pub trait IndexBuilder {
    fn cf_names(&self) -> Vec<String>;
}

pub trait Entity {
    fn key(&self) -> Vec<u8>;
    fn encode(&self) -> Vec<u8>;
}

pub trait Operations<E: Entity> {
    fn get(&self, id: u64) -> Result<Option<E>, Box<dyn Error>>;
    fn put(&self, e: &E) -> Result<u64, Box<dyn Error>>;
    fn delete(&self, e: &E) -> Result<bool, Box<dyn Error>>;
}

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

impl Error for ErrBadDbPath {}

fn check_path(path: &str) -> Result<&Path, Box<dyn Error>> {
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

static CF_SYSTEM: &str = "cf.system";
static SEQ_KEY: &str = "sequence";

// CF for storing type information.
static CF_SYSTEM_TYPES: &str = "cf.system.types";

fn all_column_families(builder: &dyn IndexBuilder) -> Vec<String> {
    let mut indexes = builder.cf_names();
    indexes.push(CF_SYSTEM.to_string());
    indexes.push(CF_SYSTEM_TYPES.to_string());
    trace!("all_column_families: {:?}", indexes);
    return indexes;
}

pub fn init(info: &dyn DbInfo, builder: &dyn IndexBuilder) -> Result<(), Box<dyn Error>> {
    let path = info.path();
    let options = info.options();
    trace!("Init path={:?}", path);

    let mut db = open_db(info, builder)?;
    let found = DB::list_cf(&options, path).unwrap_or(vec![]);

    let want = all_column_families(builder);
    trace!("Want column families: {:?}", want);
    trace!("Found column families: {:?}", found);

    for c in want.iter() {
        if found.iter().find(|cf| cf.as_str() == c).is_none() {
            // create a new ColumnFamily
            info!("Creating column family {:?}", c);
            let create_cf = db.create_cf(c, &options)?;
            info!("Creating column family {:?}, result = {:?}", c, create_cf);
        } else {
            info!("Found column family {:?}", c);
        }
    }

    Ok(())
}

// TODO - Optimize this a bit more so that opening the database simply
// opens all the column families, without creating them (do that in "init").
fn open_db(info: &dyn DbInfo, builder: &dyn IndexBuilder) -> Result<Database, Box<dyn Error>> {
    trace!("open_db path={}", info.path());
    let options = info.options();
    match DB::open_cf(
        &options,
        check_path(info.path())?,
        all_column_families(builder),
    ) {
        Ok(db) => Ok(db),
        Err(e) => {
            error!("Error opening db: {:?}", e);
            Err(Box::new(e))
        }
    }
}

// Returns the type code by checking a global lookup table of names;
// creates new entry if name is not found.
fn type_code(db: &Database, name: &String) -> Result<u64, Box<dyn Error>> {
    let type_code: u64;
    let cf = db.cf_handle(CF_SYSTEM_TYPES).unwrap();
    match db.get_cf(cf, name.as_bytes()) {
        Ok(Some(v)) => {
            let le = v.try_into().unwrap_or_else(|v: Vec<u8>| {
                panic!("Expected a Vec of length {} but it was {}", 8, v.len())
            });
            type_code = u64::from_le_bytes(le) + 1;
            trace!("type_code read: {}", type_code);
        }
        Ok(None) => {
            type_code = 1;
        }
        Err(e) => {
            error!("Error retrieving value for {}: {}", name, e);
            return Err(Box::new(e));
        }
    }

    // update the id before returning the value
    match db.put_cf(cf, name.as_bytes(), type_code.to_le_bytes().to_vec()) {
        Ok(()) => Ok(type_code),
        Err(e) => {
            error!("Error updating sequence {}", SEQ_KEY);
            Err(Box::new(e))
        }
    }
}

fn next_id(db: &Database) -> Result<u64, Box<dyn Error>> {
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
            error!("Error retrieving value for {}: {}", SEQ_KEY, e);
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

pub fn put_node<'a>(info: &'a dyn DbInfo, node: &'a mut Node) -> Result<&'a Node, Box<dyn Error>> {
    trace!("node: {:?}", node);
    let mut db = open_db(info, &All)?;
    node.id = next_id(&db)?;
    node.type_code = type_code(&db, &node.type_name)?;

    // let cf = db.cf_handle(node::ById.cf_name()).unwrap();
    // let kv = node::ById.key_value(node);
    // db.put_cf(cf, kv.0, kv.1)?;

    let mut txn = Transaction::default();
    let _: Vec<_> = Node::indexes()
        .iter()
        .map(|index| index.update_entry(&mut db, &mut txn, &node))
        .collect();

    db.write(txn)?;
    Ok(node)
}

pub fn get_node(info: &dyn DbInfo, id: u64) -> Result<Option<Node>, Box<dyn Error>> {
    trace!("db: {:?}, node id: {:?}", info.path(), id);

    let db = open_db(info, &All)?;

    let cf = db.cf_handle(node::ById.cf_name()).unwrap();
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

pub fn put_edge<'a>(info: &'a dyn DbInfo, edge: &'a mut Edge) -> Result<&'a Edge, Box<dyn Error>> {
    trace!("edge: {:?}", edge);

    let mut db = open_db(info, &All)?;

    edge.id = next_id(&db)?;
    edge.type_code = type_code(&db, &edge.type_name)?;

    let mut txn = Transaction::default();
    let _: Vec<_> = Edge::indexes()
        .iter()
        .map(|index| index.update_entry(&mut db, &mut txn, &edge))
        .collect();

    db.write(txn)?;
    Ok(edge)
}

#[allow(dead_code)]
pub fn get_edge(info: &dyn DbInfo, id: u64) -> Result<Option<Edge>, Box<dyn Error>> {
    trace!("db: {:?}, edge id: {:?}", info.path(), id);

    let db = open_db(info, &All)?;

    let cf = db.cf_handle(edge::ById.cf_name()).unwrap();
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

pub fn indexes(info: &dyn DbInfo) -> Result<Vec<String>, Box<dyn Error>> {
    trace!("Indexes path={}", info.path());

    let db = open_db(info, &All)?;
    trace!("DB = {:?}", db);

    //    let cfs = DB::list_cf(&options, p).unwrap_or(vec![]);

    let options = Options::default();
    match DB::list_cf(&options, info.path()) {
        Ok(l) => Ok(l),
        Err(e) => {
            error!("Error listing column families {:?}", e);
            Err(Box::new(e))
        }
    }
}

pub fn put(info: &dyn DbInfo, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    trace!("Put path={}, key={}, value={}", info.path(), key, value);

    let db = open_db(info, &All)?;
    trace!("DB = {:?}", db);

    let cf = db.cf_handle(kv::StringKV.cf_name()).unwrap();
    let kv = kv::StringKV.key_value(&(key.to_string(), value.to_string()));
    match db.put_cf(cf, kv.0, kv.1) {
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
) -> Result<Option<String>, Box<dyn Error>> {
    trace!("Get path={}, key={}", info.path(), key);

    let db = open_db(info, &All)?;
    trace!("DB = {:?}", db);

    let cf = db.cf_handle(kv::StringKV.cf_name()).unwrap();
    match db.get_cf(cf, key.as_bytes()) {
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
            error!("Error retrieving value for {}: {}", key, e);
            Err(Box::new(e))
        }
    }
}

pub fn delete(info: &dyn DbInfo, key: &str) -> Result<(), Box<dyn Error>> {
    trace!("Delete path={}, key={}", info.path(), key);
    let db = open_db(info, &All)?;
    trace!("DB = {:?}", db);

    let cf = db.cf_handle(kv::StringKV.cf_name()).unwrap();
    match db.delete_cf(cf, key.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("Error retrieving value for {}: {}", key, e);
            return Err(Box::new(e));
        }
    }
}

pub fn list(info: &dyn DbInfo, key: &str, visitor: &dyn VisitKV) -> Result<(), Box<dyn Error>> {
    trace!("List path={}, key={}", info.path(), key);
    let db = open_db(info, &All)?;
    trace!("DB = {:?}", db);

    let cf = db.cf_handle(kv::StringKV.cf_name()).unwrap();
    let iter = db.iterator_cf(cf, IteratorMode::From(key.as_bytes(), Direction::Forward));
    for item in iter {
        let (k, v) = item.unwrap();
        visitor.visit(&k, &v);
    }
    Ok(())
}
