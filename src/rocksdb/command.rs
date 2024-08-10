#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db::{self, HasKey, Visitor};
use crate::rocksdb::edge::EdgePrinter;
use crate::rocksdb::graph::{Edge, Node};
use crate::rocksdb::index::Index;
use crate::rocksdb::node;
use crate::rocksdb::node::NodePrinter;
use crate::rocksdb::All;

use crate::rocksdb::db::OperationsBuilder;

use clap::{Args as clapArgs, Subcommand};
use rocksdb::Options;
use std::default::Default;

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
    Counter(CounterArgs),
    Index(IndexCommand),
    Node(NodeCommand),
    Edge(EdgeCommand),
}

#[derive(Debug, clapArgs)]
pub struct InitArgs {}

#[derive(Debug, clapArgs)]
pub struct CounterArgs {
    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct IndexCommand {
    #[clap(subcommand)]
    verb: IndexVerb,
}

#[derive(Debug, Subcommand)]
pub enum IndexVerb {
    All,
    Dump(IndexArgs),
}

#[derive(Debug, clapArgs)]
pub struct IndexArgs {
    /// The key
    index: String,
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
    List(NodeListArgs),
    ListEdges(NodeListEdgesArgs),
    ByName(NodeByNameArgs),
    First(NodeFirstArgs),
}

#[derive(Debug, clapArgs)]
pub struct NodePutArgs {
    /// The name of the node
    name: String,

    /// The type name of the node
    #[clap(long = "type")]
    type_name: Option<String>,

    /// The description of the node
    description: Option<String>,
    /// The id of the node
    #[clap(long = "id")]
    id: Option<u64>,
}

#[derive(Debug, clapArgs)]
pub struct NodeGetArgs {
    /// The id of the node
    id: u64,
}

#[derive(Debug, clapArgs)]
pub struct NodeByNameArgs {
    /// Match bytes
    match_string: String,
    /// How many to return.  1 == exact match.
    n: u32,
}

#[derive(Debug, clapArgs)]
pub struct NodeFirstArgs {
    /// The name of the index
    index: String,

    /// Match bytes
    match_string: String,
}

#[derive(Debug, clapArgs)]
pub struct NodeListArgs {
    /// The start of id range
    start_id: u64,

    /// How many to list
    n: u32,
}

#[derive(Debug, clapArgs)]
pub struct NodeListEdgesArgs {
    /// The direction of the edge
    #[clap(long = "to")]
    to: bool,

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
    List(EdgeListArgs),
}

#[derive(Debug, clapArgs)]
pub struct EdgePutArgs {
    /// The head id
    head: u64,
    /// The tail id
    tail: u64,
    /// The name of the edge / relation
    name: String,

    /// The type name of the edge
    #[clap(long = "type")]
    type_name: Option<String>,

    /// The id of the node
    #[clap(long = "id")]
    id: Option<u64>,
}

#[derive(Debug, clapArgs)]
pub struct EdgeGetArgs {
    /// The id of the node
    id: u64,
}

#[derive(Debug, clapArgs)]
pub struct EdgeListArgs {
    /// List all edges starting at id
    start_id: u64,
    /// How many to list
    n: u32,
}

struct BytesVisitor(u32);
impl db::Visitor<(Box<[u8]>, Box<[u8]>)> for BytesVisitor {
    fn visit(&mut self, kv: (Box<[u8]>, Box<[u8]>)) -> bool {
        println!("[{:?}] | {:?}", kv.0, kv.1);
        self.0 = self.0 - 1;
        self.0 > 0
    }
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match &cmd.verb {
        Verb::Init(args) => {
            trace!("Called start: {:?}", args);
            let result = db::init(&cmd.db, &All);
            trace!("Result: {:?}", result);
        }
        Verb::Counter(args) => {
            trace!("Called count: {:?}", args);
            let database = db::open_db(&cmd.db, &All).unwrap();
            let mut counters = db::default_counters(&database);
            let mut counter = counters.get(args.key.as_str()).unwrap();
            let result = counter.get();
            counter.inc();
            let mut txn = db::Transaction::default();
            let commit = match counters.update(&mut txn, &counter) {
                Ok(()) => true,
                Err(_) => false,
            };
            if commit {
                match database.write(txn) {
                    Ok(()) => trace!("committed"),
                    Err(e) => error!("Error: {:?}", e),
                }
            }
            trace!("Result: {:?}", result);
        }
        Verb::Index(indexcmd) => {
            trace!("Called Indexes: {:?}", indexcmd);

            match &indexcmd.verb {
                IndexVerb::All => {
                    let result = db::indexes(&cmd.db);
                    trace!("Result: {:?}", result);
                    println!("{:?}", result);
                }
                IndexVerb::Dump(args) => {
                    trace!("Dump index content: {:?}", args);
                    let result =
                        db::list_index(&cmd.db, &args.index, &mut BytesVisitor(u32::max_value()));
                    trace!("Result: {:?}", result);
                }
            }
        }

        Verb::Node(ncmd) => {
            trace!("Called node: {:?}", ncmd);
            let database = db::open_db(&cmd.db, &All).unwrap();

            match &ncmd.verb {
                NodeVerb::Put(args) => {
                    let id: u64 = match args.id {
                        Some(v) => v,
                        None => 0,
                    };
                    let mut node = Node {
                        id,
                        type_name: match &args.type_name {
                            Some(v) => v.to_string(),
                            None => "entity".to_string(),
                        },
                        type_code: 0,
                        name: args.name.clone(),
                        cas: "".into(),
                    };

                    let mut ops = Node::operations(&database);
                    let result = ops.put(&mut node);

                    info!("Result: {:?}", result);
                }
                NodeVerb::Get(args) => {
                    let ops = Node::operations(&database);
                    let result = ops.get(Node::id_from(args.id));

                    trace!("Result: {:?}", result);
                    match result {
                        Ok(Some(node)) => {
                            let mut p = NodePrinter(1);
                            p.visit(node);
                        }
                        Ok(None) => {
                            info!("not found");
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                }
                NodeVerb::List(args) => {
                    info!("List {:?} nodes from id={:?}", args.n, args.start_id,);
                    let ops = Node::operations(&database);
                    match ops.visit(Node::id_from(args.start_id), Box::new(NodePrinter(args.n))) {
                        Ok(()) => {
                            info!("Done");
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                }
                NodeVerb::ByName(args) => {
                    trace!("Lookup by index: {:?}", args);
                    let ops = Node::operations(&database);
                    match ops.match_bytes(
                        &node::ByName.cf_name().to_string(),
                        args.match_string.as_bytes(),
                        args.n,
                    ) {
                        Ok(matches) => {
                            let c = matches.len();
                            for n in matches {
                                println!("{:?}", n);
                            }
                            info!("Done ({:?})", c);
                        }
                        Err(e) => error!("Error: {:?}", e),
                    };
                }
                NodeVerb::First(args) => {
                    trace!("First in index: {:?}", args);
                    let ops = Node::operations(&database);
                    match ops.first(&args.index, args.match_string.as_bytes()) {
                        Ok(Some(obj)) => println!("{:?}", obj),
                        Ok(None) => println!("Not found."),
                        Err(e) => error!("Error: {:?}", e),
                    }
                }
                NodeVerb::ListEdges(args) => {
                    info!("TODO - List edges of node {:?}, args {:?}", args.id, args);
                }
            }
        }
        Verb::Edge(ncmd) => {
            trace!("Called edge: {:?}", cmd);
            let database = db::open_db(&cmd.db, &All).unwrap();
            match &ncmd.verb {
                EdgeVerb::Put(args) => {
                    let id: u64 = match args.id {
                        Some(v) => v,
                        None => 0,
                    };
                    let mut edge = Edge {
                        id,
                        head: args.head,
                        tail: args.tail,
                        type_name: match &args.type_name {
                            Some(v) => v.to_string(),
                            None => "relation".to_string(),
                        },
                        type_code: 0,
                        name: args.name.clone(),
                        cas: String::from(""),
                    };
                    let mut ops = Edge::operations(&database);
                    let result = ops.put(&mut edge);

                    info!("Result: {:?}", result);
                }
                EdgeVerb::Get(args) => {
                    let ops = Edge::operations(&database);
                    let result = ops.get(Edge::id_from(args.id));

                    trace!("Result: {:?}", result);
                    match result {
                        Ok(Some(edge)) => {
                            let mut p = EdgePrinter;
                            p.visit(edge);
                        }
                        Ok(None) => {
                            info!("not found");
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                }
                EdgeVerb::List(args) => {
                    trace!("List edges: {:?}", args);
                    let ops = Edge::operations(&database);
                    match ops.visit(Edge::id_from(args.start_id), Box::new(EdgePrinter)) {
                        Ok(()) => {
                            info!("Done");
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                }
            }
        }
    }
}
