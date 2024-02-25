#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use prost::Message; // need the trait to encode protobuf

use crate::rocksdb::db;
use crate::rocksdb::graph::Edge;
use crate::rocksdb::index::{Index, Indexes};

use std::error::Error;
use std::io::Cursor;

impl db::HasKey<u64> for Edge {
    fn key(&self) -> Option<u64> {
        if self.id > 0 {
            Some(self.id)
        } else {
            None
        }
    }
}

impl db::Entity for Edge {
    const TYPE: &'static str = "Edge";
    fn as_bytes(&self) -> Vec<u8> {
        return self.encode_to_vec();
    }
    fn from_bytes(bytes: &[u8]) -> Result<Edge, Box<dyn Error>> {
        match Edge::decode(Cursor::new(bytes)) {
            Ok(edge) => Ok(edge),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl db::OperationsBuilder<Edge> for Edge {
    fn operations(db: &db::Database) -> Box<dyn db::Operations<Edge> + '_> {
        Box::new(db::OperationsImpl::<u64, Edge> {
            db: db,
            custom: &OperationsImpl {},
        })
    }
}

struct OperationsImpl {}

impl db::OperationsCustom<u64, Edge> for OperationsImpl {
    fn value_index(&self) -> &dyn Index<Edge> {
        &ById
    }
    fn indexes(&self) -> Vec<Box<dyn Index<Edge>>> {
        Edge::indexes()
    }
    fn before_put(&self, db: &db::Database, edge: &mut Edge) -> Result<(), Box<dyn Error>> {
        if edge.id == 0 {
            edge.id = db::next_id(db)?;
        }
        edge.type_code = db::type_code(db, &edge.type_name)?;
        Ok(())
    }
    fn from_bytes(&self, buff: &[u8]) -> Result<Edge, Box<dyn Error>> {
        Ok(Message::decode(&buff[..])?)
    }
}

impl Indexes<Edge> for Edge {
    fn indexes() -> Vec<Box<dyn Index<Edge>>> {
        return vec![
            // By Id,
            Box::new(ById),
            // By type code
            Box::new(ByType),
            // By head, tail
            Box::new(ByHeadTail),
            // By tail, head
            Box::new(ByTailHead),
        ];
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ById;

#[derive(Debug, Clone, PartialEq)]
pub struct ByType;

#[derive(Debug, Clone, PartialEq)]
pub struct ByHeadTail;

#[derive(Debug, Clone, PartialEq)]
pub struct ByTailHead;

impl std::fmt::Debug for dyn Index<Edge> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.cf_name()).finish()
    }
}

impl Index<Edge> for ById {
    fn cf_name(&self) -> &'static str {
        "index.edge.id"
    }
    fn key_value(&self, e: &Edge) -> (Vec<u8>, Vec<u8>) {
        use crate::rocksdb::db::Entity;
        use crate::rocksdb::db::HasKey;
        (e.id().as_bytes(), e.as_bytes())
    }
}

impl Index<Edge> for ByType {
    fn cf_name(&self) -> &'static str {
        "index.edge.type"
    }
    fn key_value(&self, e: &Edge) -> (Vec<u8>, Vec<u8>) {
        (
            e.type_code.to_le_bytes().to_vec(),
            e.id.to_le_bytes().to_vec(),
        )
    }
}

impl Index<Edge> for ByHeadTail {
    fn cf_name(&self) -> &'static str {
        "index.edge.head-tail"
    }
    fn key_value(&self, e: &Edge) -> (Vec<u8>, Vec<u8>) {
        let mut key = e.head.to_le_bytes().to_vec();
        key.extend(e.tail.to_le_bytes().to_vec());
        return (key, e.id.to_le_bytes().to_vec());
    }
}

impl Index<Edge> for ByTailHead {
    fn cf_name(&self) -> &'static str {
        "index.edge.tail-head"
    }
    fn key_value(&self, e: &Edge) -> (Vec<u8>, Vec<u8>) {
        let mut key = e.tail.to_le_bytes().to_vec();
        key.extend(e.head.to_le_bytes().to_vec());
        return (key, e.id.to_le_bytes().to_vec());
    }
}

#[test]
fn test_using_edge_indexes() {
    let mut cfs: Vec<&str> = Vec::<&str>::new();

    for i in Edge::indexes().iter() {
        cfs.push(i.cf_name());
        println!("index = {:?}, cf = {:?}", i, i.cf_name());
    }

    assert_eq!(cfs.len(), Edge::indexes().len());

    // Test using the helpers
    println!("cf = {:?}", ById.cf_name());
    println!(
        "kv = {:?}",
        ByType.key_value(&Edge {
            id: 3u64,
            head: 1u64,
            tail: 2u64,
            type_name: "foo".into(),
            type_code: 3u64,
            name: "foo".into(),
            doc: vec![],
        })
    );
}

#[test]
fn test_build_edge_keys() {
    // head, tail ids
    let k = (1u64, 2u64);

    let b = (k.0.to_le_bytes(), k.1.to_le_bytes());

    // combine [u8; 8] into Vec[u8]
    let mut kk: Vec<u8> = b.0.to_vec();
    kk.extend(b.1.to_vec());
    println!("first, second = {:?}", kk);

    let buff: &Vec<u8> = &kk;
    println!("buff = {:?}", buff);

    assert_eq!(kk, *buff);

    // make sure number of octets line up
    assert_eq!(std::mem::size_of::<u64>() * 2, buff.len());

    let parts = (
        &buff[0..std::mem::size_of::<u64>()],
        &buff[std::mem::size_of::<u64>()..buff.len()],
    );

    // compare the slices; useful for terminating prefix scans of edges
    assert!(b.0 == parts.0);

    let parsed = (
        u64::from_le_bytes(parts.0.try_into().unwrap()),
        u64::from_le_bytes(parts.1.try_into().unwrap()),
    );
    println!("read keys {:?}, {:?}", parsed.0, parsed.1);

    assert_eq!(k, parsed);
}
