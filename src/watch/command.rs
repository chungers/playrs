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
    Run(RunArgs),
    Stop(StopArgs),
}

#[derive(Debug, clapArgs)]
pub struct RunArgs {
    /// The filesystem path to watch
    pub path: Option<String>,

    /// True to watch recursively from given path
    #[clap(short)]
    pub recursive: bool,
}

#[derive(Debug, clapArgs)]
pub struct StopArgs {
    /// True to kill
    #[clap(short)]
    pub kill: bool,
}
