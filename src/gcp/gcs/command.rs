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
    Ls(LsArgs),
    Put(PutArgs),
    Get(GetArgs),
}

#[derive(Debug, clapArgs)]
pub struct LsArgs {
    /// The bucket
    bucket: String,
}

#[derive(Debug, clapArgs)]
pub struct PutArgs {
    /// The bucket
    bucket: String,
}

#[derive(Debug, clapArgs)]
pub struct GetArgs {
    /// The bucket
    bucket: String,
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match &cmd.verb {
        Verb::Ls(args) => {
            trace!("Called ls {:?}", args);
        }
        Verb::Put(args) => {
            trace!("Called put {:?}", args);
        }
        Verb::Get(args) => {
            trace!("Called get {:?}", args);
        }
    }
}
