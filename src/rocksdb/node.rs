#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use prost::Message; // need the trait to encode protobuf

use crate::rocksdb::db;
use crate::rocksdb::graph::Node;
use crate::rocksdb::index::{Index, Indexes, Queries};

use std::convert::TryInto;
use std::error::Error;

impl db::Entity for Node {
    fn key(&self) -> Vec<u8> {
        return self.id.to_le_bytes().to_vec();
    }
    fn encode(&self) -> Vec<u8> {
        return self.encode_to_vec();
    }
}

struct Operations<'a> {
    db: &'a db::Database,
}

impl db::OperationsBuilder<u64, Node> for Node {
    fn operations<'a>(db: &db::Database) -> Box<dyn db::Operations<u64, Node> + '_> {
        return Box::new(Operations { db: db });
    }
}

impl db::Operations<u64, Node> for Operations<'_> {
    fn get(&self, id: u64) -> Result<Option<Node>, Box<dyn Error>> {
        let cf = self.db.cf_handle(ById.cf_name()).unwrap();
        match self.db.get_cf(cf, u64::to_le_bytes(id)) {
            Ok(Some(bytes)) => {
                trace!("Found node id = {:?} found {:?}", id, bytes);
                let decoded: Node = Message::decode(&bytes[..])?;
                Ok(Some(decoded))
            }
            Ok(None) => {
                trace!("No node id = {:?} found", id);
                Ok(None)
            }
            Err(e) => {
                error!("Error: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
    fn put(&mut self, node: &mut Node) -> Result<u64, Box<dyn Error>> {
        if node.id == 0 {
            node.id = db::next_id(self.db)?;
        }
        node.type_code = db::type_code(self.db, &node.type_name)?;

        let mut txn = db::Transaction::default();
        let _: Vec<_> = Node::indexes()
            .iter()
            .map(|index| index.update_entry(self.db, &mut txn, &node))
            .collect();

        self.db.write(txn)?;

        Ok(node.id)
    }
    fn delete(&self, node: &Node) -> Result<bool, Box<dyn Error>> {
        trace!("Deleting {:?}", node);
        Ok(true)
    }
}

impl Indexes<Node> for Node {
    fn indexes() -> Vec<Box<dyn Index<Node>>> {
        return vec![
            // By Id,
            Box::new(ById),
            // By type code
            Box::new(ByType),
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

impl Index<Node> for ById {
    fn cf_name(&self) -> &'static str {
        return "index.node.id";
    }
}

impl Index<Node> for ByType {
    fn cf_name(&self) -> &'static str {
        return "index.node.type";
    }
    fn key_value(&self, n: &Node) -> (Vec<u8>, Vec<u8>) {
        return (
            n.type_code.to_le_bytes().to_vec(),
            n.id.to_le_bytes().to_vec(),
        );
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
            doc: vec![],
        })
    );
}

#[test]
fn test_using_node_operations() {}
