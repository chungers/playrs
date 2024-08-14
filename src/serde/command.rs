use clap::{Args as clapArgs, Subcommand, ValueEnum};

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
    Encode(EncodeArgs),
    Decode(DecodeArgs),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Encoding {
    Yaml,
    Json,
}

#[derive(Debug, clapArgs)]
pub struct EncodeArgs {
    /// Encoding to use
    #[clap(long)]
    encoding: Encoding,

    /// x coordinate for Point
    x: f64,

    /// y coordinate for Point
    y: f64,
}

#[derive(Debug, clapArgs)]
pub struct DecodeArgs {
    /// Encoding to use
    #[clap(long)]
    encoding: Encoding,

    #[clap(long = "stdin", short = 's')]
    stdin: bool,
}

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    match cmd.verb.as_ref().unwrap() {
        Verb::Encode(args) => {
            trace!("Encode: {:?}", args);
            match &args.encoding {
                Encoding::Json => crate::serde::json::encode(args.x, args.y),
                Encoding::Yaml => crate::serde::yaml::encode(args.x, args.y),
            }
        }
        Verb::Decode(args) => {
            trace!("Dncode: {:?}", args);
            match &args.encoding {
                Encoding::Json => crate::serde::json::decode(),
                Encoding::Yaml => crate::serde::yaml::decode(),
            }
        }
    }
}
