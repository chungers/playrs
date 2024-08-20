#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use prost::Message; // need the trait to encode protobuf

#[allow(unused_imports)]
use crate::rocksdb::db::{self, Visitor};
use crate::rocksdb::graph::Edge;
use crate::rocksdb::index::{Index, Indexes};

use std::error::Error;
use std::io::Cursor;
use time::OffsetDateTime;

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
        self.encode_to_vec()
    }
    fn from_bytes(_key: &[u8], bytes: &[u8]) -> Result<Edge, Box<dyn Error>> {
        Ok(Edge::decode(Cursor::new(bytes))?)
    }
}

impl db::OperationsBuilder<Edge> for Edge {
    fn operations(db: &db::Database) -> Box<dyn db::Operations<Edge> + '_> {
        db::entity_operations::<u64, Edge>(db, Box::new(IndexHelper {}))
    }
}

// TODO - Refactor to use generics
pub struct EdgePrinter(pub u32);
impl db::Visitor<Edge> for EdgePrinter {
    fn visit(&mut self, entity: Edge) -> bool {
        println!("{:?}", entity);
        self.0 = self.0 - 1;
        self.0 > 0
    }
}

#[derive(Debug, PartialEq)]
pub struct EdgeCollector<'a> {
    list: &'a mut Vec<Edge>,
    max: usize,
}

impl<'a> EdgeCollector<'a> {
    pub fn new(list: &'a mut Vec<Edge>, max: usize) -> Self {
        Self { list, max }
    }
    pub fn len(&self) -> usize {
        self.list.len()
    }
}

impl<'a> db::Visitor<Edge> for EdgeCollector<'a> {
    fn visit(&mut self, entity: Edge) -> bool {
        if self.max == self.list.len() {
            return false;
        }
        self.list.push(entity);
        self.max = self.max - 1;
        self.max > 0
    }
}

impl db::Visitor<Edge> for Vec<Edge> {
    fn visit(&mut self, entity: Edge) -> bool {
        self.push(entity);
        true
    }
}

#[test]
fn test_using_edge_collector() {
    fn static_dispatch<V: db::Visitor<Edge>>(v: &mut V) {
        assert!(v.visit(Edge::default()))
    }
    fn dyn_dispatch(mut ptr: Box<dyn db::Visitor<Edge> + '_>) {
        assert!(ptr.visit(Edge::default()));
    }

    let mut buff: Vec<Edge> = vec![];
    static_dispatch(&mut buff);
    assert_eq!(1, buff.len());
    static_dispatch(&mut buff);
    assert_eq!(2, buff.len());

    let collector2 = EdgeCollector::new(&mut buff, 100);
    dyn_dispatch(Box::new(collector2));
    assert_eq!(3, buff.len());

    let collector3 = EdgeCollector::new(&mut buff, 100);
    assert_eq!(3, collector3.len());
    dyn_dispatch(Box::new(collector3));
    assert_eq!(4, buff.len());

    let mut collector4 = EdgeCollector::new(&mut buff, 4);
    assert!(!collector4.visit(Edge::default()));
}

struct IndexHelper {}

impl db::IndexHelper<u64, Edge> for IndexHelper {
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
        // TODO - This should be set by the db if Entity has a trait for setting
        // the timestamp.  In general, Entity should have Id and Timestamp
        if edge.ts_nano.len() == 0 {
            edge.ts_nano = OffsetDateTime::now_utc()
                .unix_timestamp_nanos()
                .to_le_bytes()
                .to_vec();
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
            // By Name
            Box::new(ByName),
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
pub struct ByName;

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

impl Index<Edge> for ByName {
    fn cf_name(&self) -> &'static str {
        "index.edge.name"
    }
    fn key_value(&self, e: &Edge) -> (Vec<u8>, Vec<u8>) {
        (e.name.as_bytes().to_vec(), e.id.to_le_bytes().to_vec())
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
            type_name: "".into(),
            type_code: 3u64,
            name: "".into(),
            ts_nano: vec![],
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
