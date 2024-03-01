#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::index::Index;
use crate::rocksdb::kv;
use crate::rocksdb::All;

use rocksdb::{
    DBWithThreadMode, Direction, IteratorMode, SingleThreaded, WriteBatchWithTransaction, DB,
};

use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;

pub trait DbInfo {
    fn path(&self) -> &str;
    fn options(&self) -> rocksdb::Options;
}

pub type Database = DBWithThreadMode<SingleThreaded>;
pub type Transaction = WriteBatchWithTransaction<false>;

#[derive(Debug, Clone, PartialEq)]
pub struct Id<E: Entity + ?Sized> {
    key: Vec<u8>,
    phantom: PhantomData<E>,
}

impl<E: Entity + ?Sized> Id<E> {
    pub fn as_bytes(&self) -> Vec<u8> {
        self.key.to_vec()
    }
}

pub trait KeyCodec {
    fn encode_key(&self) -> Vec<u8>;
    fn decode_key(b: Vec<u8>) -> Self;
}

pub trait HasKey<K: KeyCodec> {
    fn key(&self) -> Option<K>; // Raw key; key may not be set
    fn id_from(v: K) -> Id<Self>
    where
        Self: Entity,
        Self: Sized,
    {
        return Id::<Self> {
            key: v.encode_key(),
            phantom: PhantomData::<Self>,
        };
    }
    fn id(&self) -> Id<Self>
    where
        Self: Entity,
        Self: Sized,
    {
        return Id::<Self> {
            key: match self.key() {
                Some(v) => v.encode_key(),
                None => vec![],
            },
            phantom: PhantomData::<Self>,
        };
    }
}

pub trait Entity: std::cmp::PartialEq + std::fmt::Debug {
    const TYPE: &'static str;
    fn as_bytes(&self) -> Vec<u8>;
    fn from_bytes(key: &[u8], bytes: &[u8]) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

impl KeyCodec for u64 {
    fn encode_key(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
    fn decode_key(buff: Vec<u8>) -> u64 {
        u64::from_le_bytes(buff.try_into().unwrap())
    }
}

// Implementations for typed entities must implement this
// trait as a builder for getting an Operations trait implementation
// which has CRUD methods.
pub trait OperationsBuilder<E: Entity> {
    fn operations(db: &Database) -> Box<dyn Operations<E> + '_>;
}

pub trait Operations<E: Entity> {
    fn get(&self, id: Id<E>) -> Result<Option<E>, Box<dyn Error>>;
    fn put(&mut self, e: &mut E) -> Result<Id<E>, Box<dyn Error>>;
    fn delete(&self, e: &E) -> Result<bool, Box<dyn Error>>;
}

pub trait IndexHelper<K: KeyCodec, E: Entity + HasKey<K>> {
    fn value_index(&self) -> &dyn Index<E>;
    fn indexes(&self) -> Vec<Box<dyn Index<E>>>;
    fn before_put(&self, db: &Database, e: &mut E) -> Result<(), Box<dyn Error>>;
    fn from_bytes(&self, buff: &[u8]) -> Result<E, Box<dyn Error>>;
}

pub fn entity_operations<K: KeyCodec + 'static, E: Entity + HasKey<K> + 'static>(
    db: &Database,
    ops: Box<dyn IndexHelper<K, E>>,
) -> Box<dyn Operations<E> + '_> {
    Box::new(OperationsImpl::<K, E> {
        db: db,
        custom: ops,
    })
}

struct OperationsImpl<'a, K: KeyCodec, E: Entity + HasKey<K>> {
    db: &'a Database,
    custom: Box<dyn IndexHelper<K, E> + 'a>,
}

impl<'a, K: KeyCodec, E: Entity + HasKey<K>> Operations<E> for OperationsImpl<'_, K, E> {
    fn get(&self, id: Id<E>) -> Result<Option<E>, Box<dyn Error>> {
        let cf = self
            .db
            .cf_handle(self.custom.value_index().cf_name())
            .unwrap();
        match self.db.get_cf(cf, id.key) {
            Ok(Some(bytes)) => Ok(Some(self.custom.from_bytes(&bytes[..])?)),
            Ok(None) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }
    fn put(&mut self, o: &mut E) -> Result<Id<E>, Box<dyn Error>> {
        self.custom.before_put(self.db, o)?;

        // Get the old version before the update so we can reindex if needed.
        let reindex = match self.get(o.id()) {
            Ok(Some(current)) => current != *o,
            Ok(None) => false,
            Err(e) => {
                return Err(e);
            }
        };
        trace!("Reindex = {}, o={:?}", reindex, o);

        let mut txn = Transaction::default();
        let _: Vec<_> = self
            .custom
            .indexes()
            .iter()
            .map(|index| index.update_entry(self.db, &mut txn, &o))
            .collect();
        self.db.write(txn)?;
        Ok(o.id())
    }
    fn delete(&self, _o: &E) -> Result<bool, Box<dyn Error>> {
        todo!()
    }
}

pub trait VisitKV {
    fn visit(&self, _: &[u8], _: &[u8]);
}

pub trait IndexBuilder {
    fn cf_names(&self) -> Vec<String>;
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

pub fn init(info: &dyn DbInfo, builder: &dyn IndexBuilder) -> Result<Database, Box<dyn Error>> {
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

    Ok(db)
}

// TODO - Optimize this a bit more so that opening the database simply
// opens all the column families, without creating them (do that in "init").
pub fn open_db(info: &dyn DbInfo, builder: &dyn IndexBuilder) -> Result<Database, Box<dyn Error>> {
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
pub fn type_code(db: &Database, name: &String) -> Result<u64, Box<dyn Error>> {
    let type_code: u64;
    let cf = db.cf_handle(CF_SYSTEM_TYPES).unwrap();
    match db.get_cf(cf, name.as_bytes()) {
        Ok(Some(v)) => {
            let le = v.try_into().unwrap_or_else(|v: Vec<u8>| {
                panic!("Expected a Vec of length {} but it was {}", 8, v.len())
            });
            type_code = u64::from_le_bytes(le);
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

    // BUG: type code is always 1 since we are not counting the number of types/ rows
    // TODO: update a counter to track the number of rows in the types table/cf.

    // update the id before returning the value
    match db.put_cf(cf, name.as_bytes(), type_code.to_le_bytes().to_vec()) {
        Ok(()) => Ok(type_code),
        Err(e) => {
            error!("Error updating sequence {}", SEQ_KEY);
            Err(Box::new(e))
        }
    }
}

pub fn next_id(db: &Database) -> Result<u64, Box<dyn Error>> {
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

pub fn indexes(
    info: &dyn DbInfo,
    //    builder: &dyn IndexBuilder,
) -> Result<Vec<String>, Box<dyn Error>> {
    trace!("Indexes path={}", info.path());
    // let db = open_db(info, builder)?;
    // trace!("DB = {:?}", db);
    match DB::list_cf(&info.options(), info.path()) {
        Ok(l) => Ok(l),
        Err(e) => {
            error!("Error listing column families {:?}", e);
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
