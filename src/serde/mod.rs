use clap::Args as clapArgs;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, trace, warn};

#[derive(clapArgs)]
pub struct Args {
    name: Option<String>,

    /// x coordinate for Point
    #[clap(short)]
    x: Option<i32>,

    /// y coordinate for Point
    #[clap(short)]
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

//pub fn serde(name: &Option<String>) {
pub fn serde(args: &Args) {
    trace!("serde was used with arg: {:?}", args.name);

    let mut point = Point {
        ..Default::default()
    };
    match args.x {
        Some(v) => point.x = v,
        None => {}
    }
    match args.y {
        Some(v) => point.y = v,
        None => {}
    }

    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(&point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    info!("serialized = {}", serialized);

    // Convert the JSON string back to a Point.
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    info!("deserialized JSON = {:?}", deserialized);
}
