use std::error::Error;
use tempfile::tempdir;

use crate::rocksdb::db::{DbInfo, OperationsBuilder};
use crate::rocksdb::graph::Node;
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
fn test_node_id_assignment() -> Result<(), Box<dyn Error>> {
    // Setup a temporary database
    let db_info = TestDbInfo::new();
    let db = crate::rocksdb::db::init(&db_info, &All)?;

    // Create a test node with ID 0 (should be auto-assigned)
    let mut node = Node {
        id: 0,
        type_name: "TestNode".into(),
        type_code: 0, // Will be auto-assigned
        name: "TestNodeName".into(),
        ts_nano: vec![],
    };

    // Get operations for Node
    let mut node_ops = Node::operations(&db);

    // Insert the node
    let node_id = node_ops.put(&mut node)?;

    // Assert that the node has an ID assigned (non-zero)
    assert_ne!(
        node.id, 0,
        "Node should have an ID assigned after insertion"
    );

    // Assert that the node has been assigned an ID and it's valid
    assert!(node.id > 0, "Node ID should be a valid non-zero value");

    // Verify the node exists with the assigned ID
    let retrieved_node = node_ops.get(node_id.clone())?;
    assert!(
        retrieved_node.is_some(),
        "Node should exist after insertion"
    );
    assert_eq!(
        retrieved_node.unwrap().id,
        node.id,
        "Retrieved node should have the same ID"
    );

    Ok(())
}

#[test]
fn test_node_delete() -> Result<(), Box<dyn Error>> {
    // Setup a temporary database
    let db_info = TestDbInfo::new();
    let db = crate::rocksdb::db::init(&db_info, &All)?;

    // Create a test node
    let mut node = Node {
        id: 0,
        type_name: "TestNode".into(),
        type_code: 0,
        name: "TestNodeName".into(),
        ts_nano: vec![],
    };

    // Get operations for Node
    let mut node_ops = Node::operations(&db);

    // Insert the node
    let node_id = node_ops.put(&mut node)?;

    // Verify the node exists
    let retrieved_node = node_ops.get(node_id.clone())?;
    assert!(
        retrieved_node.is_some(),
        "Node should exist after insertion"
    );

    // Delete the node
    let delete_result = node_ops.delete(&node)?;
    assert!(
        delete_result,
        "Delete operation should return true for successful deletion"
    );

    // Verify the node no longer exists
    let retrieved_after_delete = node_ops.get(node_id.clone())?;
    assert!(
        retrieved_after_delete.is_none(),
        "Node should not exist after deletion"
    );

    Ok(())
}
