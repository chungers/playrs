#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;
use crate::rocksdb::index::Index;

use std::error::Error;

impl db::Entity for (String, String) {
    fn key(&self) -> Vec<u8> {
        return self.0.as_bytes().to_vec();
    }
    fn encode(&self) -> Vec<u8> {
        return self.1.as_bytes().to_vec();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringKV;

struct Operations<'a> {
    db: &'a db::Database,
}

impl db::OperationsBuilder<(String, String)> for (String, String) {
    fn operations<'a>(db: &db::Database) -> Box<dyn db::Operations<(String, String)> + '_> {
        return Box::new(Operations { db: db });
    }
}

impl db::Operations<(String, String)> for Operations<'_> {
    fn get(&self, id: u64) -> Result<Option<(String, String)>, Box<dyn Error>> {
        Ok(None)
    }

    fn put(&mut self, kv: &mut (String, String)) -> Result<u64, Box<dyn Error>> {
        Ok(1u64)
    }

    fn delete(&self, kv: &(String, String)) -> Result<bool, Box<dyn Error>> {
        Ok(true)
    }
}

impl std::fmt::Debug for dyn Index<(String, String)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.cf_name()).finish()
    }
}

impl Index<(String, String)> for StringKV {
    fn cf_name(&self) -> &'static str {
        return "index.kv";
    }
    fn key_value(&self, kv: &(String, String)) -> (Vec<u8>, Vec<u8>) {
        return (kv.0.as_bytes().to_vec(), kv.1.as_bytes().to_vec());
    }
}
