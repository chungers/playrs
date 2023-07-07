use super::gcs;

use clap::{Args as clapArgs, Subcommand};

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[derive(Debug, clapArgs)]
pub struct Commands {
    #[clap(subcommand)]
    pub service: Service,
}

#[derive(Debug, Subcommand)]
pub enum Service {
    /// GCS examples
    Gcs(gcs::command::Command),
}

pub fn go(cmd: &Commands) {
    trace!("Running commands: {:?}", cmd);

    match &cmd.service {
        Service::Gcs(args) => {
            trace!("Called gcs: {:?}", args);
            gcs::command::go(args);
        }
    }
}
