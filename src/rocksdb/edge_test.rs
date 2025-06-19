use std::error::Error;
use tempfile::tempdir;

use crate::rocksdb::db::{DbInfo, OperationsBuilder};
use crate::rocksdb::graph::{Edge, Node};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_delete() -> Result<(), Box<dyn Error>> {
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

        // Insert the nodes first
        let mut node_ops = Node::operations(&db);
        let _head_id = node_ops.put(&mut head_node)?;
        let _tail_id = node_ops.put(&mut tail_node)?;

        // Create a test edge using the populated node IDs
        let mut edge = Edge {
            id: 0,
            type_name: "TestEdge".into(),
            type_code: 0,
            name: "TestEdgeName".into(),
            head: head_node.id,
            tail: tail_node.id,
            ts_nano: vec![],
        };

        // Get operations for Edge
        let mut edge_ops = Edge::operations(&db);

        // Insert the edge
        let edge_id = edge_ops.put(&mut edge)?;

        // Verify the edge exists
        let retrieved_edge = edge_ops.get(edge_id.clone())?;
        assert!(
            retrieved_edge.is_some(),
            "Edge should exist after insertion"
        );
        assert_eq!(retrieved_edge.unwrap().name, "TestEdgeName");

        // Delete the edge
        let delete_result = edge_ops.delete(&edge)?;
        assert!(
            delete_result,
            "Delete operation should return true for successful deletion"
        );

        // Verify the edge no longer exists
        let retrieved_after_delete = edge_ops.get(edge_id.clone())?;
        assert!(
            retrieved_after_delete.is_none(),
            "Edge should not exist after deletion"
        );

        Ok(())
    }

    #[test]
    fn test_edge_delete_nonexistent() -> Result<(), Box<dyn Error>> {
        // Setup a temporary database
        let db_info = TestDbInfo::new();
        let db = crate::rocksdb::db::init(&db_info, &All)?;

        // Create a dummy edge that doesn't exist in the database
        let edge = Edge {
            id: 999,
            type_name: "NonExistentEdge".into(),
            type_code: 0,
            name: "NonExistentEdgeName".into(),
            head: 1,
            tail: 2,
            ts_nano: vec![],
        };

        // Get operations for Edge
        let mut edge_ops = Edge::operations(&db);

        // Try to delete the non-existent edge
        let delete_result = edge_ops.delete(&edge)?;
        assert!(
            !delete_result,
            "Delete operation should return false for non-existent edge"
        );

        Ok(())
    }

    #[test]
    fn test_edge_delete_multiple() -> Result<(), Box<dyn Error>> {
        // Setup a temporary database
        let db_info = TestDbInfo::new();
        let db = crate::rocksdb::db::init(&db_info, &All)?;

        // Create nodes for the edges
        let mut head_node = Node {
            id: 0,
            type_name: "SourceNode".into(),
            type_code: 0,
            name: "SourceNodeName".into(),
            ts_nano: vec![],
        };

        let mut tail_node1 = Node {
            id: 0,
            type_name: "TargetNode1".into(),
            type_code: 0,
            name: "TargetNode1Name".into(),
            ts_nano: vec![],
        };

        let mut tail_node2 = Node {
            id: 0,
            type_name: "TargetNode2".into(),
            type_code: 0,
            name: "TargetNode2Name".into(),
            ts_nano: vec![],
        };

        // Insert the nodes
        let mut node_ops = Node::operations(&db);
        let _head_id = node_ops.put(&mut head_node)?;
        let _tail_id1 = node_ops.put(&mut tail_node1)?;
        let _tail_id2 = node_ops.put(&mut tail_node2)?;

        // Create multiple edges using the populated node IDs
        let mut edge1 = Edge {
            id: 0,
            type_name: "ConnectsTo".into(),
            type_code: 0,
            name: "Edge1".into(),
            head: head_node.id,
            tail: tail_node1.id,
            ts_nano: vec![],
        };

        let mut edge2 = Edge {
            id: 0,
            type_name: "ConnectsTo".into(),
            type_code: 0,
            name: "Edge2".into(),
            head: head_node.id,
            tail: tail_node2.id,
            ts_nano: vec![],
        };

        // Get operations for Edge
        let mut edge_ops = Edge::operations(&db);

        // Insert both edges
        let edge_id1 = edge_ops.put(&mut edge1)?;
        let edge_id2 = edge_ops.put(&mut edge2)?;

        // Verify both edges exist
        let retrieved_edge1 = edge_ops.get(edge_id1.clone())?;
        let retrieved_edge2 = edge_ops.get(edge_id2.clone())?;
        assert!(
            retrieved_edge1.is_some(),
            "Edge1 should exist after insertion"
        );
        assert!(
            retrieved_edge2.is_some(),
            "Edge2 should exist after insertion"
        );

        // Delete the first edge
        let delete_result1 = edge_ops.delete(&edge1)?;
        assert!(
            delete_result1,
            "Delete operation should return true for edge1"
        );

        // Verify first edge is deleted but second still exists
        let retrieved_after_delete1 = edge_ops.get(edge_id1.clone())?;
        let retrieved_edge2_still = edge_ops.get(edge_id2.clone())?;
        assert!(
            retrieved_after_delete1.is_none(),
            "Edge1 should not exist after deletion"
        );
        assert!(
            retrieved_edge2_still.is_some(),
            "Edge2 should still exist after deleting edge1"
        );

        // Delete the second edge
        let delete_result2 = edge_ops.delete(&edge2)?;
        assert!(
            delete_result2,
            "Delete operation should return true for edge2"
        );

        // Verify second edge is also deleted
        let retrieved_after_delete2 = edge_ops.get(edge_id2.clone())?;
        assert!(
            retrieved_after_delete2.is_none(),
            "Edge2 should not exist after deletion"
        );

        Ok(())
    }
}
