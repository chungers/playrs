#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

pub mod command;
mod db;
mod edge;
mod index;
mod kv;
mod node;

#[path = "rocksdb.graph.v1.rs"] // generated by protoc
mod graph;

mod server;

#[derive(Debug, Clone, PartialEq)]
pub struct All;

use crate::rocksdb::index::Index; // use here to access to cf_name() method on index.

impl db::IndexBuilder for All {
    fn cf_names(&self) -> Vec<String> {
        vec![
            kv::StringKV.cf_name().into(),
            node::ById.cf_name().into(),
            edge::ById.cf_name().into(),
            node::ByType.cf_name().into(),
            edge::ByType.cf_name().into(),
            edge::ByHeadTail.cf_name().into(),
        ]
    }
}
