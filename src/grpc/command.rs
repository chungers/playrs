#[path = "server.rs"]
mod server;

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
    message: String,
}

pub fn start(args: &StartArgs) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting server {:?}", args.name);

    return server::start(&args.port);
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match cmd.verb.as_ref().unwrap() {
        Verb::Start(args) => {
            trace!("Called start: {:?}", args);
            start(args);
        }
        Verb::Stop(args) => {
            trace!("Called stop: {:?}", args);
        }
        Verb::Call(args) => {
            trace!("Called call: {:?}", args);
        }
    }
}
