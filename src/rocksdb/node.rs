#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use prost::Message; // need the trait to encode protobuf

use crate::rocksdb::db;
use crate::rocksdb::graph::Node;
use crate::rocksdb::hash;
use crate::rocksdb::index::{Index, Indexes};

use std::error::Error;
use std::io::Cursor;
use time::OffsetDateTime;

impl db::HasKey<u64> for Node {
    fn key(&self) -> Option<u64> {
        if self.id > 0 {
            Some(self.id)
        } else {
            None
        }
    }
}

impl db::Entity for Node {
    const TYPE: &'static str = "Node";
    fn as_bytes(&self) -> Vec<u8> {
        self.encode_to_vec()
    }
    fn from_bytes(_key: &[u8], bytes: &[u8]) -> Result<Node, Box<dyn Error>> {
        Ok(Node::decode(Cursor::new(bytes))?)
    }
}

impl db::OperationsBuilder<Node> for Node {
    fn operations(db: &db::Database) -> Box<dyn db::Operations<Node> + '_> {
        db::entity_operations::<u64, Node>(db, Box::new(IndexHelper {}))
    }
}

impl Node {
    /// Compute SHA-256 hash of the node's name
    pub fn name_hash(&self) -> String {
        hash::compute_sha256_hash(&self.name)
    }
}

pub struct NodePrinter(pub usize);

impl db::Visitor<Node> for NodePrinter {
    fn visit(&mut self, entity: Node) -> bool {
        println!("{:?}", entity);
        self.0 = self.0 - 1;
        self.0 > 0
    }
}
struct IndexHelper {}

impl db::IndexHelper<u64, Node> for IndexHelper {
    fn value_index(&self) -> &dyn Index<Node> {
        &ById
    }
    fn indexes(&self) -> Vec<Box<dyn Index<Node>>> {
        Node::indexes()
    }
    fn before_put(&self, db: &db::Database, node: &mut Node) -> Result<(), Box<dyn Error>> {
        if node.id == 0 {
            node.id = db::next_id(db)?;
        }
        // TODO - This should be set by the db if Entity has a trait for setting
        // the timestamp.  In general, Entity should have Id and Timestamp
        if node.ts_nano.len() == 0 {
            node.ts_nano = OffsetDateTime::now_utc()
                .unix_timestamp_nanos()
                .to_le_bytes()
                .to_vec();
        }
        node.type_code = db::type_code(db, &node.type_name)?;
        Ok(())
    }
    fn from_bytes(&self, buff: &[u8]) -> Result<Node, Box<dyn Error>> {
        Ok(Message::decode(&buff[..])?)
    }
}

impl Indexes<Node> for Node {
    fn indexes() -> Vec<Box<dyn Index<Node>>> {
        return vec![
            // By Id,
            Box::new(ById),
            // By type code
            Box::new(ByType),
            // By name
            Box::new(ByName),
            // By name_hash
            Box::new(ByNameHash),
        ];
    }
}

impl std::fmt::Debug for dyn Index<Node> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(self.cf_name()).finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ById;

#[derive(Debug, Clone, PartialEq)]
pub struct ByType;

#[derive(Debug, Clone, PartialEq)]
pub struct ByName;

#[derive(Debug, Clone, PartialEq)]
pub struct ByNameHash;

impl Index<Node> for ById {
    fn cf_name(&self) -> &'static str {
        "index.node.id"
    }
    fn key_value(&self, e: &Node) -> (Vec<u8>, Vec<u8>) {
        use crate::rocksdb::db::Entity;
        use crate::rocksdb::db::HasKey;
        (e.id().as_bytes(), e.as_bytes())
    }
}

impl Index<Node> for ByType {
    fn cf_name(&self) -> &'static str {
        "index.node.type"
    }
    fn key_value(&self, n: &Node) -> (Vec<u8>, Vec<u8>) {
        (
            n.type_code.to_le_bytes().to_vec(),
            n.id.to_le_bytes().to_vec(),
        )
    }
}

impl Index<Node> for ByName {
    fn cf_name(&self) -> &'static str {
        "index.node.name"
    }
    fn key_value(&self, n: &Node) -> (Vec<u8>, Vec<u8>) {
        trace!(
            "key_value = [{:?}, {:?}={:?}], obj={:?}",
            n.name,
            n.id,
            n.id.to_le_bytes().to_vec(),
            n
        );
        //       (n.name.encode_to_vec(), n.id.to_le_bytes().to_vec())
        (n.name.as_bytes().to_vec(), n.id.to_le_bytes().to_vec())
    }
}

impl Index<Node> for ByNameHash {
    fn cf_name(&self) -> &'static str {
        "index.node.name_hash"
    }
    fn append_only(&self) -> bool {
        true
    }
    fn key_value(&self, n: &Node) -> (Vec<u8>, Vec<u8>) {
        trace!(
            "key_value = [{:?}, {:?}={:?}], obj={:?}",
            n.name,
            n.id,
            n.id.to_le_bytes().to_vec(),
            n
        );
        //       (n.name.encode_to_vec(), n.id.to_le_bytes().to_vec())
        (
            n.name_hash().as_bytes().to_vec(),
            n.id.to_le_bytes().to_vec(),
        )
    }
}

#[test]
fn test_using_node_indexes() {
    let mut cfs: Vec<&str> = Vec::<&str>::new();

    for i in Node::indexes().iter() {
        cfs.push(i.cf_name());
        println!("index = {:?}, cf = {:?}", i, i.cf_name());
    }

    assert_eq!(cfs.len(), Node::indexes().len());

    // Test using the helpers
    println!("cf = {:?}", ById.cf_name());
    println!(
        "kv = {:?}",
        ByType.key_value(&Node {
            id: 1u64,
            type_name: "".into(),
            type_code: 2u64,
            name: "".into(),
            ts_nano: vec![],
        })
    );
}

#[test]
fn test_using_node_id() {
    let node = Node {
        id: 1u64,
        type_name: "".into(),
        type_code: 2u64,
        name: "".into(),
        ts_nano: vec![],
    };

    use db::HasKey;
    println!("Got id = {:?}", node.id());
    println!("Id from raw = {:?}", Node::id_from(1u64));
}

#[test]
fn test_node_name_hash() {
    let node = Node {
        id: 1u64,
        type_name: "entity".into(),
        type_code: 2u64,
        name: "test_node".into(),
        ts_nano: vec![],
    };

    let hash = node.name_hash();
    println!("Node name hash: {}", hash);
    assert_eq!(
        hash,
        "55f3cc97541a60372fc6eb73be4a9ca056a36d937844875d43570ca8b1dc1e30"
    );

    // Test with empty name
    let empty_node = Node {
        id: 2u64,
        type_name: "entity".into(),
        type_code: 2u64,
        name: "".into(),
        ts_nano: vec![],
    };

    let empty_hash = empty_node.name_hash();
    assert_eq!(
        empty_hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}
