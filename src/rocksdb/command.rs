#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;
use crate::rocksdb::db::HasKey;
use crate::rocksdb::graph::{Edge, Node};
use crate::rocksdb::kv;
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
    Counter(CounterArgs),
    Index(IndexCommand),
    Kv(KvCommand),
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
pub struct KvCommand {
    #[clap(subcommand)]
    verb: KvVerb,
}

#[derive(Debug, Subcommand)]
pub enum KvVerb {
    Put(KvPutArgs),
    Get(KvGetArgs),
    Delete(KvDeleteArgs),
    List(KvListArgs),
}

#[derive(Debug, clapArgs)]
pub struct KvPutArgs {
    /// The key
    key: String,

    /// The value
    value: String,
}

#[derive(Debug, clapArgs)]
pub struct KvGetArgs {
    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct KvDeleteArgs {
    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct KvListArgs {
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
    List(NodeListArgs),
    ListEdges(NodeListEdgesArgs),
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
pub struct NodeListArgs {
    /// The start of id range
    start_id: u64,

    /// The end of id range
    end_id: u64,
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
    /// List all edges to the node id
    #[clap(long = "to")]
    to: bool,
    /// List by the id of the node
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
            let result = db::init(&cmd.db, &All);
            trace!("Result: {:?}", result);
        }
        Verb::Counter(args) => {
            trace!("Called count: {:?}", args);
            let database = db::open_db(&cmd.db, &All).unwrap();
            let counters = db::default_counters(&database);
            let counter = counters.get(args.key.as_str()).unwrap();
            let result = counter.get();
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
                    let result = db::list_index(&cmd.db, &args.index, &visit);
                    trace!("Result: {:?}", result);
                }
            }
        }

        Verb::Kv(kvcmd) => {
            trace!("Called kv: {:?}", kvcmd);
            let database = db::open_db(&cmd.db, &All).unwrap();

            match &kvcmd.verb {
                KvVerb::Put(args) => {
                    trace!("Called put: {:?}", args);
                    let mut ops = kv::StringKV::operations(&database);
                    let result = ops.put(&mut (args.key.to_string(), args.value.to_string()));
                    trace!("Result: {:?}", result);
                }
                KvVerb::Get(args) => {
                    trace!("Called get: {:?}", args);
                    let ops = kv::StringKV::operations(&database);
                    let result = ops.get(<(String, String)>::id_from(args.key.to_string()));
                    trace!("Result: {:?}", result);
                }
                KvVerb::Delete(args) => {
                    trace!("Called delete: {:?}", args);
                    let result = db::delete(&cmd.db, &args.key);
                    trace!("Result: {:?}", result);
                }
                KvVerb::List(args) => {
                    trace!("Called list: {:?}", args);
                    let result = db::list(&cmd.db, &args.prefix, &visit);
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
                        id: id,
                        type_name: match &args.type_name {
                            Some(v) => v.to_string(),
                            None => "entity".to_string(),
                        },
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
                    let result = ops.get(Node::id_from(args.id));

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
                NodeVerb::List(args) => {
                    info!(
                        "TODO - List nodes from range [{:?}, {:?}]",
                        args.start_id, args.end_id
                    );
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
                        id: id,
                        head: args.head,
                        tail: args.tail,
                        type_name: match &args.type_name {
                            Some(v) => v.to_string(),
                            None => "relation".to_string(),
                        },
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
                    let result = ops.get(Edge::id_from(args.id));

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
                EdgeVerb::List(args) => {
                    info!("TODO - List edges by {:?}", args);
                }
            }
        }
    }
}
