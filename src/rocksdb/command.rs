// Test including a module that's at the parent level. not public
#[path = "db.rs"] // using path isn't ideal
mod db;

use clap::{Args as clapArgs, Subcommand};

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[derive(Debug, clapArgs)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct Command {
    #[clap(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    Start(StartArgs),
    Stop(StopArgs),
    Init(InitArgs),
    Put(PutArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    List(ListArgs),
}

#[derive(Debug, clapArgs)]
pub struct StartArgs {
    /// Path to the db file
    path: String,

    /// The domain socket name
    port: String,

    /// Fork the process.  The command will return, and the server runs in background.
    #[clap(short, long)]
    fork: bool,
}

#[derive(Debug, clapArgs)]
pub struct StopArgs {
    /// The Port
    port: String,

    /// Kills the process
    #[clap(short)]
    kill: bool,
}

#[derive(Debug, clapArgs)]
pub struct InitArgs {
    /// The DB path
    path: String,
}

impl db::DbInfo for InitArgs {
    fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, clapArgs)]
pub struct PutArgs {
    /// The DB path
    path: String,

    /// The key
    key: String,

    /// The value
    value: String,
}

impl db::DbInfo for PutArgs {
    fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, clapArgs)]
pub struct GetArgs {
    /// The DB path
    path: String,
    /// The key
    key: String,
}

impl db::DbInfo for GetArgs {
    fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, clapArgs)]
pub struct DeleteArgs {
    /// The DB path
    path: String,
    /// The key
    key: String,
}

impl db::DbInfo for DeleteArgs {
    fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, clapArgs)]
pub struct ListArgs {
    /// The DB path
    path: String,
    /// The key
    prefix: String,
}

impl db::DbInfo for ListArgs {
    fn path(&self) -> &str {
        &self.path
    }
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match &cmd.verb {
        Verb::Start(args) => {
            trace!("Called start: {:?}", args);
        }
        Verb::Stop(args) => {
            trace!("Called stop: {:?}", args);
        }
        Verb::Init(args) => {
            trace!("Called start: {:?}", args);
            let result = db::init(args);
            trace!("Result: {:?}", result);
        }
        Verb::Put(args) => {
            trace!("Called put: {:?}", args);
            let result = db::put(args, &args.key, &args.value);
            trace!("Result: {:?}", result);
        }
        Verb::Get(args) => {
            trace!("Called get: {:?}", args);
            let result = db::get(args, &args.key);
            trace!("Result: {:?}", result);
        }
        Verb::Delete(args) => {
            trace!("Called delete: {:?}", args);
            let result = db::delete(args, &args.key);
            trace!("Result: {:?}", result);
        }
        Verb::List(args) => {
            trace!("Called list: {:?}", args);
            let result = db::list(args, &args.prefix);
            trace!("Result: {:?}", result);
        }
    }
}
