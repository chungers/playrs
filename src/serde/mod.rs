use clap::Args as clapArgs;

use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[derive(clapArgs, Debug)]
pub struct Args {
    /// Format to use (e.g. JSON or YAML)
    #[clap(short)]
    format: Option<String>,

    /// x coordinate for Point
    x: Option<i32>,

    /// y coordinate for Point
    y: Option<i32>,
}

impl Default for Point {
    fn default() -> Point {
        Point { x: 0, y: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

pub mod command;
pub mod json;
pub mod yaml;
