use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: f64,
    y: f64,
}

impl Default for Point {
    fn default() -> Point {
        Point { x: 0., y: 0. }
    }
}

pub mod command;
pub mod json;
pub mod yaml;
