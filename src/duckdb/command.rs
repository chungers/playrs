use clap::{ArgEnum, Args as clapArgs, Subcommand};

use crate::duckdb::db;
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
    Example,
    Create(CreateArgs),
    Read(ReadArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

#[derive(Debug, clapArgs)]
pub struct CreateArgs {
    /// Name of the person
    name: String,
}

#[derive(Debug, clapArgs)]
pub struct ReadArgs {}

#[derive(Debug, clapArgs)]
pub struct UpdateArgs {
    /// Name of the person
    name: String,
}

#[derive(Debug, clapArgs)]
pub struct DeleteArgs {}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match cmd.verb.as_ref().unwrap() {
        Verb::Example => {
            trace!("Example");
            db::example();
        }
        Verb::Create(args) => {
            trace!("Create: {:?}", args);
        }
        Verb::Read(args) => {
            trace!("Read: {:?}", args);
        }
        Verb::Update(args) => {
            trace!("Update: {:?}", args);
        }
        Verb::Delete(args) => {
            trace!("Delete: {:?}", args);
        }
    }
}
