#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::graph::{Edge, Node};

use std::convert::TryInto;

pub trait Index<E> {
    fn cf_name(&self) -> &'static str;
    fn key_value(&self, obj: &E) -> (Vec<u8>, Vec<u8>);
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeById;
#[derive(Debug, Clone, PartialEq)]
pub struct NodeByName;
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeById;
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeByHeadTail;
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeByName;

impl Index<Node> for NodeById {
    fn cf_name(&self) -> &'static str {
        return "index.node.id";
    }
    fn key_value(&self, n: &Node) -> (Vec<u8>, Vec<u8>) {
        return (n.id.to_le_bytes().to_vec(), n.id.to_le_bytes().to_vec());
    }
}

impl Index<Node> for NodeByName {
    fn cf_name(&self) -> &'static str {
        return "index.node.name";
    }
    fn key_value(&self, n: &Node) -> (Vec<u8>, Vec<u8>) {
        return (n.name.as_bytes().to_vec(), n.id.to_le_bytes().to_vec());
    }
}

impl Index<Edge> for EdgeById {
    fn cf_name(&self) -> &'static str {
        return "index.edge.id";
    }
    fn key_value(&self, e: &Edge) -> (Vec<u8>, Vec<u8>) {
        return (e.id.to_le_bytes().to_vec(), e.id.to_le_bytes().to_vec());
    }
}

impl Index<Edge> for EdgeByHeadTail {
    fn cf_name(&self) -> &'static str {
        return "index.edge.head-tail";
    }
    fn key_value(&self, e: &Edge) -> (Vec<u8>, Vec<u8>) {
        let mut key = e.head.to_le_bytes().to_vec();
        key.extend(e.tail.to_le_bytes().to_vec());
        return (key, e.id.to_le_bytes().to_vec());
    }
}

impl Index<Edge> for EdgeByName {
    fn cf_name(&self) -> &'static str {
        return "index.edge.name";
    }
    fn key_value(&self, e: &Edge) -> (Vec<u8>, Vec<u8>) {
        return (e.name.as_bytes().to_vec(), e.id.to_le_bytes().to_vec());
    }
}

#[test]
fn test_build_indexes() {
    let idx: Vec<Box<dyn Index<Node>>> = vec![Box::new(NodeById), Box::new(NodeByName)];
    for i in idx.iter() {
        println!("{:?}", i.cf_name());
    }
}

#[test]
fn test_build_edge_key() {
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
