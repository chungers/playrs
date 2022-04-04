
use clap::{Subcommand, Args as clapArgs};
use std::fmt;


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

#[derive(clapArgs)]
pub struct RunArgs {

    /// The filesystem path to watch
    pub path: Option<String>,

    /// True to watch recursively from given path
    #[clap(short)]
    pub recursive: bool,
}

#[derive(clapArgs)]
pub struct StopArgs {

    /// True to kill
    #[clap(short)]
    pub kill: bool,
}

impl fmt::Debug for RunArgs {
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Run")
        .field("path", &self.path)
        .field("recursive", &self.recursive)
        .finish()
    }
}

impl fmt::Debug for StopArgs {
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Stop")
        .field("kill", &self.kill)
        .finish()
    }
}
