#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db::{self, HasKey, Visitor};
use crate::rocksdb::edge::{self, EdgeCollector, EdgePrinter};
use crate::rocksdb::graph::{Edge, Id, Node};
use crate::rocksdb::index::Index;
use crate::rocksdb::node;
use crate::rocksdb::node::NodePrinter;
use crate::rocksdb::All;

use crate::rocksdb::db::OperationsBuilder;

use clap::{Args as clapArgs, Subcommand};
use rocksdb::Options;
use std::default::Default;

#[derive(Debug, Clone, clapArgs, PartialEq, Eq)]
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
    ByName(NodeByNameArgs),
    First(NodeFirstArgs),
}

#[derive(Debug, clapArgs)]
pub struct NodePutArgs {
    /// The name of the node
    name: String,

    /// The type name of the node
    #[clap(long = "is")]
    type_name: Option<String>,

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
    n: usize,
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
    n: usize,
}

#[derive(Debug, clapArgs)]
pub struct EdgeCommand {
    #[clap(subcommand)]
    verb: EdgeVerb,
}

#[derive(Debug, clapArgs)]
pub struct NodeNameArgs {
    name: String,

    /// Raw prints the protobuf of the edges
    #[clap(short = 'r')]
    raw: bool,
}

#[derive(Debug, Subcommand)]
pub enum EdgeVerb {
    Put(EdgePutArgs),
    Associate(EdgeRelArgs),
    Get(EdgeGetArgs),
    List(EdgeListArgs),
    From(NodeNameArgs),
    To(NodeNameArgs),
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
    #[clap(long = "is")]
    type_name: Option<String>,

    /// The id of the node
    #[clap(long = "id")]
    id: Option<u64>,
}

#[derive(Debug, clapArgs)]
pub struct EdgeRelArgs {
    /// The head name
    head: String,

    /// The name of the relation
    name: String,

    /// The tail name
    tail: String,

    /// The type name of the edge
    #[clap(long = "type")]
    type_name: Option<String>,
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

#[derive(Debug, clapArgs)]
pub struct EdgeFromArgs {
    /// The head name
    head: String,

    /// The name of the relation
    relation: String,

    /// The tail name
    tail: String,

    /// The type name of the edge
    #[clap(long = "type")]
    type_name: Option<String>,
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
                        id: Some(Id { value: id }),
                        type_name: match &args.type_name {
                            Some(v) => v.to_string(),
                            None => "entity".to_string(),
                        },
                        type_code: 0,
                        name: args.name.clone(),
                        ts_nano: vec![],
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
                        Ok(()) => trace!("Done"),
                        Err(e) => error!("Error: {:?}", e),
                    }
                }
                NodeVerb::ByName(args) => {
                    trace!("Lookup by index: {:?}", args);
                    let ops = Node::operations(&database);
                    match ops.match_bytes(
                        &node::ByName.cf_name().to_string(),
                        args.match_string.as_bytes().to_vec(),
                        Box::new(NodePrinter(args.n)),
                    ) {
                        Ok(()) => trace!("Done."),
                        Err(e) => error!("Error: {:?}", e),
                    }
                }
                NodeVerb::First(args) => {
                    trace!("First in index: {:?}", args);
                    let ops = Node::operations(&database);
                    match ops.exact(&args.index, args.match_string.as_bytes()) {
                        Ok(Some(obj)) => println!("{:?}", obj),
                        Ok(None) => println!("Not found."),
                        Err(e) => error!("Error: {:?}", e),
                    }
                }
            }
        }
        Verb::Edge(ncmd) => {
            trace!("Called edge: {:?}", cmd);
            let database = db::open_db(&cmd.db, &All).unwrap();
            match &ncmd.verb {
                EdgeVerb::Associate(args) => {
                    // Look up the head and tail by name
                    let node_ops = Node::operations(&database);
                    match node_ops.exact(&node::ByName.cf_name().to_string(), args.head.as_bytes())
                    {
                        Ok(Some(head)) => {
                            match node_ops
                                .exact(&node::ByName.cf_name().to_string(), args.tail.as_bytes())
                            {
                                Ok(Some(tail)) => {
                                    trace!("{:?} --{:?}-> {:?}", head, args.name, tail);
                                    let mut edge = Edge {
                                        id: Some(Id { value: 0 }),
                                        head: head.id,
                                        tail: tail.id,
                                        type_name: match &args.type_name {
                                            Some(v) => v.to_string(),
                                            None => args.name.clone(),
                                        },
                                        type_code: 0,
                                        name: args.name.clone(),
                                        ts_nano: vec![],
                                    };
                                    let mut ops = Edge::operations(&database);
                                    let result = ops.put(&mut edge);
                                    info!("Result: {:?}", result);
                                }
                                Ok(None) => error!("Node name not found: {:?}", args.tail),
                                Err(e) => error!("Error:{:?}", e),
                            }
                        }
                        Ok(None) => error!("Node name not found: {:?}", args.head),
                        Err(e) => error!("Error:{:?}", e),
                    }
                }
                EdgeVerb::Put(args) => {
                    let id: u64 = match args.id {
                        Some(v) => v,
                        None => 0,
                    };
                    let mut edge = Edge {
                        id: Some(Id { value: id }),
                        head: Some(Id { value: args.head }),
                        tail: Some(Id { value: args.tail }),
                        type_name: match &args.type_name {
                            Some(v) => v.to_string(),
                            None => args.name.clone(),
                        },
                        type_code: 0,
                        name: args.name.clone(),
                        ts_nano: vec![],
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
                            let mut p = EdgePrinter(1);
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
                    match ops.visit(Edge::id_from(args.start_id), Box::new(EdgePrinter(args.n))) {
                        Ok(()) => {
                            info!("Done");
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                }
                EdgeVerb::From(args) => {
                    trace!("Edges from {:?}", args);
                    let mut buffer: Vec<Edge> = vec![];
                    let visitor = EdgeCollector::new(&mut buffer, usize::max_value());
                    // Look up the head and tail by name
                    let node_ops = Node::operations(&database);
                    match node_ops.exact(&node::ByName.cf_name().to_string(), args.name.as_bytes())
                    {
                        Ok(Some(head)) => {
                            let edge_ops = Edge::operations(&database);
                            match edge_ops.match_bytes(
                                &edge::ByHeadTail.cf_name().to_string(),
                                head.id
                                    .unwrap_or(Id { value: 0 })
                                    .value
                                    .to_le_bytes()
                                    .to_vec(),
                                Box::new(visitor),
                            ) {
                                Ok(()) => {
                                    for f in buffer.iter() {
                                        if !args.raw {
                                            match node_ops.get(Node::id_from(
                                                f.tail.unwrap_or(Id { value: 0 }).value,
                                            )) {
                                                Ok(Some(tail)) => println!(
                                                    "{:?} {:?} {:?}",
                                                    head.name, f.name, tail.name
                                                ),
                                                Ok(None) => error!("To not found: {:?}", f.tail),
                                                Err(e) => error!("Error {:?}", e),
                                            }
                                        } else {
                                            println!("{:?}", f);
                                        }
                                    }
                                }
                                Err(e) => error!("Error: {:?}", e),
                            }
                        }
                        Ok(None) => error!("Node from {:?} not found.", args.name),
                        Err(e) => error!("Error: {:?}", e),
                    }
                }
                EdgeVerb::To(args) => {
                    trace!("Edges to {:?}", args);
                    let mut buffer: Vec<Edge> = vec![];
                    let visitor = EdgeCollector::new(&mut buffer, usize::max_value());

                    // Look up the head and tail by name
                    let node_ops = Node::operations(&database);
                    match node_ops.exact(&node::ByName.cf_name().to_string(), args.name.as_bytes())
                    {
                        Ok(Some(tail)) => {
                            let edge_ops = Edge::operations(&database);
                            match edge_ops.match_bytes(
                                &edge::ByTailHead.cf_name().to_string(),
                                tail.id
                                    .unwrap_or(Id { value: 0 })
                                    .value
                                    .to_le_bytes()
                                    .to_vec(),
                                Box::new(visitor),
                            ) {
                                Ok(()) => {
                                    for f in buffer.iter() {
                                        if !args.raw {
                                            match node_ops.get(Node::id_from(
                                                f.head.unwrap_or(Id { value: 0 }).value,
                                            )) {
                                                Ok(Some(head)) => println!(
                                                    "{:?} {:?} {:?}",
                                                    head.name, f.name, tail.name
                                                ),
                                                Ok(None) => error!("From not found: {:?}", f.head),
                                                Err(e) => error!("Error {:?}", e),
                                            }
                                        } else {
                                            println!("{:?}", f);
                                        }
                                    }
                                }
                                Err(e) => error!("Error: {:?}", e),
                            }
                        }
                        Ok(None) => error!("Node from {:?} not found.", args.name),
                        Err(e) => error!("Error: {:?}", e),
                    }
                }
            }
        }
    }
}
