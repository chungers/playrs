#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use prost::Message; // need the trait to encode protobuf

use crate::rocksdb::db;
use crate::rocksdb::graph::Node;
use crate::rocksdb::index::{Index, Indexes};

use std::error::Error;
use std::io::Cursor;

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

pub struct NodePrinter;

impl db::Visitor<Node> for NodePrinter {
    fn visit(&self, entity: Node) -> bool {
        println!("{:?}", entity);
        true
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
            type_name: "foo".into(),
            type_code: 2u64,
            name: "foo".into(),
            cas: "".into(),
        })
    );
}

#[test]
fn test_using_node_id() {
    let node = Node {
        id: 1u64,
        type_name: "foo".into(),
        type_code: 2u64,
        name: "foo".into(),
        cas: "".into(),
    };

    use db::HasKey;
    println!("Got id = {:?}", node.id());
    println!("Id from raw = {:?}", Node::id_from(1u64));
}

#[test]
fn test_using_node_operations() {}
