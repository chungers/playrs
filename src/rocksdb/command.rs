#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;
use crate::rocksdb::graph::{Edge, Node};
use crate::rocksdb::All;

use clap::{Args as clapArgs, Subcommand};
use rocksdb::Options;

#[derive(Debug, clapArgs, PartialEq, Eq)]
pub struct DbArgs {
    /// The DB path
    path: String,
}

type Err = std::num::ParseIntError;
impl DbArgs {
    fn from_str(s: &str) -> Result<Self, Err> {
        // TODO - Check path for valid rocksdb directory
        Ok(DbArgs {
            path: s.to_string(),
        })
    }
}
impl db::DbInfo for DbArgs {
    fn path(&self) -> &str {
        &self.path
    }
    fn options(&self) -> Options {
        let mut options = Options::default();
        options.set_error_if_exists(false);
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        return options;
    }
}

// Required for claps
impl std::str::FromStr for DbArgs {
    type Err = std::num::ParseIntError; // TODO - replace with better error
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

#[derive(Debug, clapArgs)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct Command {
    #[clap(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    Init(DbArgs),
    Indexes(DbArgs),
    Put(PutArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    List(ListArgs),
    Node(NodeCommand),
    Edge(EdgeCommand),
}

#[derive(Debug, clapArgs)]
pub struct PutArgs {
    /// The DB path
    db: DbArgs,

    /// The key
    key: String,

    /// The value
    value: String,
}

#[derive(Debug, clapArgs)]
pub struct GetArgs {
    /// The DB path
    db: DbArgs,

    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct DeleteArgs {
    /// The DB path
    db: DbArgs,
    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct ListArgs {
    /// The DB path
    db: DbArgs,
    /// The key
    prefix: String,
}

#[derive(Debug, clapArgs)]
pub struct NodeCommand {
    /// The DB path
    db: DbArgs,
    #[clap(subcommand)]
    verb: NodeVerb,
}

#[derive(Debug, Subcommand)]
pub enum NodeVerb {
    Put(NodePutArgs),
    Get(NodeGetArgs),
}

#[derive(Debug, clapArgs)]
pub struct NodePutArgs {
    /// The name of the node
    name: String,
    /// The description of the node
    description: Option<String>,
}

#[derive(Debug, clapArgs)]
pub struct NodeGetArgs {
    /// The id of the node
    id: u64,
}

#[derive(Debug, clapArgs)]
pub struct EdgeCommand {
    /// The DB path
    db: DbArgs,
    #[clap(subcommand)]
    verb: EdgeVerb,
}

#[derive(Debug, Subcommand)]
pub enum EdgeVerb {
    Put(EdgePutArgs),
    Get(EdgeGetArgs),
}

#[derive(Debug, clapArgs)]
pub struct EdgePutArgs {
    /// The head id
    head: u64,
    /// The tail id
    tail: u64,
    /// The name of the edge / relation
    name: String,
}

#[derive(Debug, clapArgs)]
pub struct EdgeGetArgs {
    /// The id of the node
    id: u64,
}

struct Visitor(i32);

impl db::VisitKV for Visitor {
    fn visit(&self, k: &[u8], v: &[u8]) {
        println!(
            "k={:?}, v={:?}",
            String::from_utf8(k.to_vec()).unwrap(),
            String::from_utf8(v.to_vec()).unwrap()
        );
    }
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);
    let visit = Visitor(0);

    match &cmd.verb {
        Verb::Init(args) => {
            trace!("Called start: {:?}", args);
            let result = db::init(args, &All);
            trace!("Result: {:?}", result);
        }
        Verb::Indexes(args) => {
            trace!("List Indexes (column families): {:?}", args);
            let result = db::indexes(args);
            trace!("Result: {:?}", result);
            println!("{:?}", result);
        }
        Verb::Put(args) => {
            trace!("Called put: {:?}", args);
            let result = db::put(&args.db, &args.key, &args.value);
            trace!("Result: {:?}", result);
        }
        Verb::Get(args) => {
            trace!("Called get: {:?}", args);
            let result = db::get(&args.db, &args.key, &visit);
            trace!("Result: {:?}", result);
        }
        Verb::Delete(args) => {
            trace!("Called delete: {:?}", args);
            let result = db::delete(&args.db, &args.key);
            trace!("Result: {:?}", result);
        }
        Verb::List(args) => {
            trace!("Called list: {:?}", args);
            let result = db::list(&args.db, &args.prefix, &visit);
            trace!("Result: {:?}", result);
        }
        Verb::Node(cmd) => {
            trace!("Called node: {:?}", cmd);
            match &cmd.verb {
                NodeVerb::Put(args) => {
                    let mut node = Node {
                        id: 0,
                        name: args.name.clone(),
                    };
                    let result = db::put_node(&cmd.db, &mut node);
                    info!("Result: {:?}", result);
                }
                NodeVerb::Get(args) => {
                    let result = db::get_node(&cmd.db, args.id);
                    trace!("Result: {:?}", result);
                    match result {
                        Ok(Some(node)) => {
                            info!("{:?}", node);
                        }
                        Ok(None) => {
                            info!("not found");
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
        Verb::Edge(cmd) => match &cmd.verb {
            EdgeVerb::Put(args) => {
                let mut edge = Edge {
                    id: 0,
                    head: args.head,
                    tail: args.tail,
                    name: args.name.clone(),
                };
                let result = db::put_edge(&cmd.db, &mut edge);
                info!("Result: {:?}", result);
            }
            EdgeVerb::Get(args) => {
                let result = db::get_edge(&cmd.db, args.id);
                trace!("Result: {:?}", result);
                match result {
                    Ok(Some(edge)) => {
                        info!("{:?}", edge);
                    }
                    Ok(None) => {
                        info!("not found");
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                    }
                }
            }
        },
    }
}
