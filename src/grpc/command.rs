use super::client;
use super::server;

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
    Call(CallArgs),
}

#[derive(Debug, clapArgs)]
pub struct StartArgs {
    /// The domain socket name
    port: String,

    /// An optional name for the service
    name: Option<String>,

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
pub struct CallArgs {
    port: String,
    #[clap(short, long)]
    name: String,
}

pub fn start(args: &StartArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Starting server {:?}", args.name);
    return server::start(&args.port);
}

pub fn call(args: &CallArgs) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Calling server {:?}", args);
    return client::call(&args.port, &args.name);
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match &cmd.verb {
        Verb::Start(args) => {
            trace!("Called start: {:?}", args);
            let result = start(args);
            trace!("Result: {:?}", result);
        }
        Verb::Stop(args) => {
            trace!("Called stop: {:?}", args);
        }
        Verb::Call(args) => {
            trace!("Called call: {:?}", args);
            let result = call(args);
            trace!("Result: {:?}", result);
        }
    }
}
