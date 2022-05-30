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
    pub verb: Option<Verb>,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    Start(StartArgs),
    Stop(StopArgs),
    Put(PutArgs),
    Get(GetArgs),
    Delete(DeleteArgs),
    List(ListArgs),
}

#[derive(Debug, clapArgs)]
pub struct StartArgs {
    /// The domain socket name
    port: String,

    /// Path to the db file
    path: String,

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
pub struct PutArgs {
    /// The DB path
    path: String,

    /// The key
    key: String,

    /// The value
    value: String,
}

#[derive(Debug, clapArgs)]
pub struct GetArgs {
    /// The DB path
    path: String,
    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct DeleteArgs {
    /// The DB path
    path: String,
    /// The key
    key: String,
}

#[derive(Debug, clapArgs)]
pub struct ListArgs {
    /// The DB path
    path: String,
    /// The key
    prefix: String,
}

pub fn start(args: &StartArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("NOT IMPLEMENTED Starting server {:?}", args);
    Ok(())
}

pub fn stop(args: &PutArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("NOT IMPLEMENTED Stopping server {:?}", args);
    Ok(())
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match cmd.verb.as_ref().unwrap() {
        Verb::Start(args) => {
            trace!("Called start: {:?}", args);
            let result = start(args);
            trace!("Result: {:?}", result);
        }
        Verb::Stop(args) => {
            trace!("Called stop: {:?}", args);
        }
        Verb::Put(args) => {
            trace!("Called put: {:?}", args);
            let result = db::put(&args.path, &args.key, &args.value);
            trace!("Result: {:?}", result);
        }
        Verb::Get(args) => {
            trace!("Called get: {:?}", args);
            let result = db::get(&args.path, &args.key);
            trace!("Result: {:?}", result);
        }
        Verb::Delete(args) => {
            trace!("Called delete: {:?}", args);
            let result = db::delete(&args.path, &args.key);
            trace!("Result: {:?}", result);
        }
        Verb::List(args) => {
            trace!("Called list: {:?}", args);
            let result = db::list(&args.path, &args.prefix);
            trace!("Result: {:?}", result);
        }
    }
}
