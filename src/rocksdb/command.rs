#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;
use crate::rocksdb::graph::{Edge, Node};
use crate::rocksdb::All;

use crate::rocksdb::db::OperationsBuilder;

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
#[clap(args_conflicts_with_subcommands = false)]
pub struct Command {
    /// Path of the db file. TODO(future - full db spec with config params)
    #[clap(long = "path", short = 'p')]
    pub db: DbArgs,

    #[clap(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    Init(InitArgs),
    Indexes(IndexesArgs),
    Put(PutArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    List(ListArgs),
    Node(NodeCommand),
    Edge(EdgeCommand),
}

#[derive(Debug, clapArgs)]
pub struct InitArgs {}

#[derive(Debug, clapArgs)]
pub struct IndexesArgs {}

#[derive(Debug, clapArgs)]
pub struct PutArgs {
    /// The key
    key: String,

    /// The value
    value: String,
}

#[derive(Debug, clapArgs)]
pub struct GetArgs {
    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct DeleteArgs {
    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct ListArgs {
    /// The key
    prefix: String,
}

#[derive(Debug, clapArgs)]
pub struct NodeCommand {
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

    let database = db::open_db(&cmd.db, &All).unwrap();

    match &cmd.verb {
        Verb::Init(args) => {
            trace!("Called start: {:?}", args);
            let result = db::init(&cmd.db, &All);
            trace!("Result: {:?}", result);
        }
        Verb::Indexes(args) => {
            trace!("List Indexes (column families): {:?}", args);

            let options = Options::default();
            //            let result = database.list_cf(&options, cmd.db.path());

            let result = db::indexes(&cmd.db);
            trace!("Result: {:?}", result);
            println!("{:?}", result);
        }
        Verb::Put(args) => {
            trace!("Called put: {:?}", args);
            let result = db::put(&cmd.db, &args.key, &args.value);
            trace!("Result: {:?}", result);
        }
        Verb::Get(args) => {
            trace!("Called get: {:?}", args);
            let result = db::get(&cmd.db, &args.key, &visit);
            trace!("Result: {:?}", result);
        }
        Verb::Delete(args) => {
            trace!("Called delete: {:?}", args);
            let result = db::delete(&cmd.db, &args.key);
            trace!("Result: {:?}", result);
        }
        Verb::List(args) => {
            trace!("Called list: {:?}", args);
            let result = db::list(&cmd.db, &args.prefix, &visit);
            trace!("Result: {:?}", result);
        }
        Verb::Node(ncmd) => {
            trace!("Called node: {:?}", cmd);
            match &ncmd.verb {
                NodeVerb::Put(args) => {
                    let mut node = Node {
                        id: 0,
                        type_name: args.name.clone(),
                        type_code: 0,
                        name: args.name.clone(),
                        doc: vec![],
                    };

                    let mut ops = Node::operations(&database);
                    let result = ops.put(&mut node);

                    info!("Result: {:?}", result);
                }
                NodeVerb::Get(args) => {
                    let ops = Node::operations(&database);
                    let result = ops.get(args.id);

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
        Verb::Edge(ncmd) => match &ncmd.verb {
            EdgeVerb::Put(args) => {
                let mut edge = Edge {
                    id: 0,
                    head: args.head,
                    tail: args.tail,
                    type_name: args.name.clone(),
                    type_code: 0,
                    name: args.name.clone(),
                    doc: vec![],
                };

                let mut ops = Edge::operations(&database);
                let result = ops.put(&mut edge);

                info!("Result: {:?}", result);
            }
            EdgeVerb::Get(args) => {
                let ops = Edge::operations(&database);
                let result = ops.get(args.id);

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
