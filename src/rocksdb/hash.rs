//! Hash utilities for computing SHA-256 hashes of node names and other strings.
//!
//! This module provides functionality to compute SHA-256 hashes, particularly useful
//! for hashing `node.name` values in the RocksDB graph database.
//!
//! # Examples
//!
//! ```
//! use playrs::rocksdb::hash::compute_sha256_hash;
//!
//! let hash = compute_sha256_hash("test_node");
//! println!("SHA-256: {}", hash);
//! ```
//!
//! For Node instances, you can use the `name_hash()` method:
//!
//! ```
//! let node = Node { /* ... */ };
//! let hash = node.name_hash();
//! ```

use sha2::{Digest, Sha256};

/// Compute SHA-256 hash of a string (like node.name)
///
/// # Arguments
///
/// * `input` - The string to hash
///
/// # Returns
///
/// A lowercase hexadecimal string representation of the SHA-256 hash
///
/// # Examples
///
/// ```
/// use playrs::rocksdb::hash::compute_sha256_hash;
///
/// let hash = compute_sha256_hash("test_node");
/// assert_eq!(hash, "55f3cc97541a60372fc6eb73be4a9ca056a36d937844875d43570ca8b1dc1e30");
/// ```
pub fn compute_sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Demo function showing SHA-256 hashes for various node.name values
#[allow(dead_code)]
pub fn demo_node_name_hashes() {
    println!("=== SHA-256 Hashes for node.name Values ===");

    let node_names = vec![
        "",
        "test_node",
        "entity",
        "depends-on",
        "TestNodeName",
        "index.node.name",
        "example_node_name",
        "my-node-123",
        "🚀node",
        "a very long node name with spaces and symbols !@#$%^&*()",
    ];

    for name in node_names {
        let hash = compute_sha256_hash(name);
        println!("node.name: '{}' => SHA-256: {}", name, hash);
    }
    println!("===============================================");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_name_hash() {
        // Example: if node.name = "test_node"
        let node_name = "test_node";
        let hash = compute_sha256_hash(node_name);
        println!("SHA-256 hash of node.name '{}': {}", node_name, hash);

        // Verify it produces consistent results
        assert_eq!(hash, compute_sha256_hash(node_name));
        assert_eq!(
            hash,
            "55f3cc97541a60372fc6eb73be4a9ca056a36d937844875d43570ca8b1dc1e30"
        );
    }

    #[test]
    fn test_comprehensive_node_name_hashes() {
        // Test various node.name values that might appear in the system
        assert_eq!(
            compute_sha256_hash("entity"),
            "bca3685fea8acd4e4b5c149874e1aa2bad0708e7e5ed490f3cf0702cb7a8bb56"
        );

        assert_eq!(
            compute_sha256_hash("depends-on"),
            "30eb4989ab614fb7c1545b47b91c50324d26c7c1fd04a423438656c23cd38004"
        );

        assert_eq!(
            compute_sha256_hash("TestNodeName"),
            "e47c6e7a3b2cbf1a356cd9eca20ded955346659294e34ded82ced20a3c58ebf7"
        );

        assert_eq!(
            compute_sha256_hash("index.node.name"),
            "cc8e9671643d02ed926afcb9208b3fd012986217b1c227394eccef6327155278"
        );
    }

    #[test]
    fn test_demo_hashes() {
        // Run the demo to show hash computations
        demo_node_name_hashes();
    }

    #[test]
    fn test_empty_node_name() {
        let empty_name = "";
        let hash = compute_sha256_hash(empty_name);
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_unicode_node_name() {
        let unicode_name = "🚀node";
        let hash = compute_sha256_hash(unicode_name);
        assert_eq!(
            hash,
            "f073a52083e81d0b54ecda1527733abb30d4d3ac909c525506dcf3b346d86805"
        );
    }

    #[test]
    fn test_long_node_name_with_symbols() {
        let long_name = "a very long node name with spaces and symbols !@#$%^&*()";
        let hash = compute_sha256_hash(long_name);
        assert_eq!(
            hash,
            "e2e39c0a7fdc705e6e9c4e4eb040bbca435f25ff66c6ea49aee612fc5f438e0e"
        );
    }

    #[test]
    fn test_hash_consistency() {
        let test_values = vec![
            "test1",
            "test2",
            "another_test",
            "UPPERCASE",
            "MiXeD_CaSe",
            "123456",
            "special!@#$%",
        ];

        for value in test_values {
            let hash1 = compute_sha256_hash(value);
            let hash2 = compute_sha256_hash(value);
            assert_eq!(hash1, hash2, "Hash should be consistent for '{}'", value);
            assert_eq!(hash1.len(), 64, "SHA-256 hash should be 64 characters long");
        }
    }
}
