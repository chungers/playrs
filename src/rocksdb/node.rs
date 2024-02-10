#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use prost::Message; // need the trait to encode protobuf

use crate::rocksdb::db;
use crate::rocksdb::graph::Node;
use crate::rocksdb::index::{Index, Indexes};

impl db::Entity for Node {
    fn key(&self) -> Vec<u8> {
        return self.id.to_le_bytes().to_vec();
    }
    fn encode(&self) -> Vec<u8> {
        return self.encode_to_vec();
    }
}

impl Indexes<Node> for Node {
    fn indexes() -> Vec<Box<dyn Index<Node>>> {
        return vec![
            // By Id,
            Box::new(ById),
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
pub struct ByName;

impl Index<Node> for ById {
    fn cf_name(&self) -> &'static str {
        return "index.node.id";
    }
}

impl Index<Node> for ByName {
    fn cf_name(&self) -> &'static str {
        return "index.node.name";
    }
    fn key_value(&self, n: &Node) -> (Vec<u8>, Vec<u8>) {
        return (n.name.as_bytes().to_vec(), n.id.to_le_bytes().to_vec());
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
        ByName.key_value(&Node {
            id: 1u64,
            name: "foo".into(),
        })
    );
}
