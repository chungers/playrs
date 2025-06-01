use std::error::Error;
use std::path::Path;
use tempfile::tempdir;

use crate::rocksdb::db::{
    Database, DbInfo, Entity, HasKey, Id, KeyCodec, Operations, OperationsBuilder,
};
use crate::rocksdb::graph::{Edge, Node};
use crate::rocksdb::index::{Index, Indexes};
use crate::rocksdb::All;
use rocksdb::Options;

// A helper struct to create a temporary database for testing
struct TestDbInfo {
    path: String,
}

impl TestDbInfo {
    fn new() -> Self {
        let dir = tempdir().unwrap();
        Self {
            path: dir.path().to_str().unwrap().to_string(),
        }
    }
}

impl DbInfo for TestDbInfo {
    fn path(&self) -> &str {
        &self.path
    }

    fn options(&self) -> Options {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts
    }
}

#[test]
fn test_operations_delete_node() -> Result<(), Box<dyn Error>> {
    // Setup a temporary database
    let db_info = TestDbInfo::new();
    let db = crate::rocksdb::db::init(&db_info, &All)?;

    // Create a test node
    let mut node = Node {
        id: 0, // Will be auto-assigned by the database
        type_name: "TestNode".into(),
        type_code: 0, // Will be auto-assigned
        name: "TestNodeName".into(),
        ts_nano: vec![],
    };

    // Get operations for Node
    let mut node_ops = Node::operations(&db);

    // Insert the node first
    let node_id = node_ops.put(&mut node)?;

    // Verify the node exists
    let retrieved_node = node_ops.get(node_id)?;
    assert!(
        retrieved_node.is_some(),
        "Node should exist after insertion"
    );
    assert_eq!(retrieved_node.unwrap().name, "TestNodeName");

    // Now test the delete operation
    let delete_result = node_ops.delete(&node)?;
    assert!(
        delete_result,
        "Delete operation should return true for successful deletion"
    );

    // Verify the node no longer exists
    let retrieved_after_delete = node_ops.get(node_id)?;
    assert!(
        retrieved_after_delete.is_none(),
        "Node should not exist after deletion"
    );

    Ok(())
}

#[test]
fn test_operations_delete_edge() -> Result<(), Box<dyn Error>> {
    // Setup a temporary database
    let db_info = TestDbInfo::new();
    let db = crate::rocksdb::db::init(&db_info, &All)?;

    // Create two nodes to connect with an edge
    let mut head_node = Node {
        id: 0,
        type_name: "HeadNode".into(),
        type_code: 0,
        name: "HeadNodeName".into(),
        ts_nano: vec![],
    };

    let mut tail_node = Node {
        id: 0,
        type_name: "TailNode".into(),
        type_code: 0,
        name: "TailNodeName".into(),
        ts_nano: vec![],
    };

    // Insert the nodes
    let mut node_ops = Node::operations(&db);
    let head_id = node_ops.put(&mut head_node)?.key().unwrap();
    let tail_id = node_ops.put(&mut tail_node)?.key().unwrap();

    // Create an edge connecting the nodes
    let mut edge = Edge {
        id: 0,
        type_name: "ConnectsTo".into(),
        type_code: 0,
        name: "TestEdge".into(),
        head: head_id,
        tail: tail_id,
        ts_nano: vec![],
    };

    // Get operations for Edge
    let mut edge_ops = Edge::operations(&db);

    // Insert the edge
    let edge_id = edge_ops.put(&mut edge)?;

    // Verify the edge exists
    let retrieved_edge = edge_ops.get(edge_id)?;
    assert!(
        retrieved_edge.is_some(),
        "Edge should exist after insertion"
    );
    assert_eq!(retrieved_edge.unwrap().name, "TestEdge");

    // Now test the delete operation
    let delete_result = edge_ops.delete(&edge)?;
    assert!(
        delete_result,
        "Delete operation should return true for successful deletion"
    );

    // Verify the edge no longer exists
    let retrieved_after_delete = edge_ops.get(edge_id)?;
    assert!(
        retrieved_after_delete.is_none(),
        "Edge should not exist after deletion"
    );

    Ok(())
}
