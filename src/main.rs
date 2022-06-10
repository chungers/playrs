mod grpc;
mod rocksdb;
mod serde;
mod template;
mod watch;

use clap::{Parser, Subcommand};

use tracing::Level;

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use std::io;
use std::str::FromStr;

#[derive(Parser)]
#[clap(author = "chunger",
       version,
       about = "This is a simple learning exercise in Rust.",
       long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Log levels: trace, debug, info, warn, or error
    #[clap(long, short)]
    log_level: Option<String>,

    /// Prints out verbosely
    #[clap(long, short)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// RocksDB examples
    Rocksdb(rocksdb::command::Command),

    /// gRPC examples
    GRPC(grpc::command::Command),

    /// Serde examples
    Serde(serde::Args),

    /// Template examples
    Template(template::Args),

    /// Notify (watch files) examples
    Watch(watch::command::Command),
}

fn main() {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_max_level(match cli.log_level {
            Some(s) => match Level::from_str(&s) {
                Ok(l) => l,
                Err(_) => Level::INFO,
            },
            None => Level::INFO,
        })
        .with_writer(io::stderr) // TODO - Support log rotation (see tracing-appender)
        .with_target(false) // disable targets
        .with_file(true) // display source code file paths
        .with_line_number(true) // display source code line numbers
        .init();

    trace!("starting");

    match cli.command {
        Commands::Serde(args) => {
            if cli.verbose {
                trace!("Serde...");
            }
            serde::serde(&args);
        }
        Commands::Template(args) => {
            if cli.verbose {
                trace!("Templating...");
            }
            template::template(&args);
        }
        Commands::Watch(args) => {
            if cli.verbose {
                trace!("Watching...");
            }
            watch::watch(&args);
        }
        Commands::GRPC(args) => {
            if cli.verbose {
                trace!("GRPC...");
            }
            grpc::command::go(&args);
        }
        Commands::Rocksdb(args) => {
            if cli.verbose {
                trace!("RocksDB...");
            }
            rocksdb::command::go(&args);
        }
    }
}
