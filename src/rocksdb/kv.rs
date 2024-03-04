#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;
use crate::rocksdb::db::KeyCodec;
use crate::rocksdb::index::Index;

use std::error::Error;

impl db::HasKey<String> for (String, String) {
    fn key(&self) -> Option<String> {
        if self.0.len() > 0 {
            Some(self.0.to_string())
        } else {
            None
        }
    }
}

#[test]
fn test_using_node_id() {
    let kv = ("foo".into(), "bar".into());

    use db::HasKey;
    println!("Got id = {:?}", kv.id());
    println!(
        "Id from raw = {:?}",
        <(String, String)>::id_from("foo".into())
    );
}

impl db::Entity for (String, String) {
    const TYPE: &'static str = "(String,String)";
    fn as_bytes(&self) -> Vec<u8> {
        self.1.as_bytes().to_vec()
    }
    fn from_bytes(key: &[u8], bytes: &[u8]) -> Result<(String, String), Box<dyn Error>> {
        Ok((
            String::from_utf8(key.to_vec())?,
            String::from_utf8(bytes.to_vec())?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringKV;

impl KeyCodec for String {
    fn encode_key(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    fn decode_key(buff: Vec<u8>) -> String {
        String::from_utf8(buff).unwrap()
    }
}

impl db::OperationsBuilder<(String, String)> for StringKV {
    fn operations(db: &db::Database) -> Box<dyn db::Operations<(String, String)> + '_> {
        db::entity_operations::<String, (String, String)>(db, Box::new(IndexHelper {}))
    }
}
struct IndexHelper {}

impl db::IndexHelper<String, (String, String)> for IndexHelper {
    fn value_index(&self) -> &dyn Index<(String, String)> {
        &StringKV
    }
    fn indexes(&self) -> Vec<Box<dyn Index<(String, String)>>> {
        vec![]
    }
    fn before_put(
        &self,
        _db: &db::Database,
        kv: &mut (String, String),
    ) -> Result<(), Box<dyn Error>> {
        dbg!(kv);
        Ok(())
    }
    fn from_bytes(&self, buff: &[u8]) -> Result<(String, String), Box<dyn Error>> {
        dbg!(buff);
        todo!()
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
