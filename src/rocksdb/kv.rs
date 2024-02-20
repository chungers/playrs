#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;
use crate::rocksdb::db::Key;
use crate::rocksdb::index::Index;

use std::convert::TryFrom;
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

impl Key for String {
    fn encode_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    fn decode_key(buff: Vec<u8>) -> String {
        String::from_utf8(buff).unwrap()
    }
}

struct Operations<'a> {
    db: &'a db::Database,
}

impl db::OperationsBuilder<String, (String, String)> for StringKV {
    fn operations<'a>(db: &db::Database) -> Box<dyn db::Operations<String, (String, String)> + '_> {
        return Box::new(Operations { db: db });
    }
}

impl db::Operations<String, (String, String)> for Operations<'_> {
    fn get(&self, key: String) -> Result<Option<(String, String)>, Box<dyn Error>> {
        let cf = self.db.cf_handle(StringKV.cf_name()).unwrap();
        match self.db.get_cf(cf, key.as_bytes()) {
            Ok(Some(v)) => {
                let result = String::from_utf8(v).unwrap();
                trace!("Finding '{}' returns '{}'", key, result);
                Ok(Some((key, result)))
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

    // TODO - The u64 return type doesn't work for simple string key-values.
    fn put(&mut self, pair: &mut (String, String)) -> Result<String, Box<dyn Error>> {
        let cf = self.db.cf_handle(StringKV.cf_name()).unwrap();
        let kv = StringKV.key_value(&(pair.0.to_string(), pair.1.to_string()));
        trace!("put_cf: {:?}, ({:?}, {:?})", StringKV.cf_name(), kv.0, kv.1);
        match self.db.put_cf(cf, kv.0, kv.1) {
            Ok(()) => Ok(pair.0.to_string()),
            Err(e) => {
                error!("Error put for ({:?}: {}", pair, e);
                Err(Box::new(e))
            }
        }
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
