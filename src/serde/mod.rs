use clap::ArgEnum;
use clap::Args as clapArgs;

use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum Encoding {
    Yaml,
    Json,
}

#[derive(clapArgs, Debug)]
pub struct Args {
    /// Encoding to use
    #[clap(long, arg_enum)]
    encoding: Option<Encoding>,

    /// x coordinate for Point
    #[clap(short = 'X')]
    x: Option<i32>,

    /// y coordinate for Point
    #[clap(short = 'Y')]
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

fn serde_json(point: &Point) {
    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    info!("serialized = {}", serialized);

    // Convert the JSON string back to a Point.
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    info!("deserialized JSON = {:?}", deserialized);
}

fn serde_yaml(point: &Point) {
    // Convert the Point to a YAML string.
    let serialized = serde_yaml::to_string(point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    info!("serialized = {}", serialized);

    // Convert the YAML string back to a Point.
    let deserialized: Point = serde_yaml::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    info!("deserialized YAML = {:?}", deserialized);
}

pub fn serde(args: &Args) {
    trace!("serde was used with arg: {:?}", args);

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

    match args.encoding {
        Some(v) => match v {
            Encoding::Json => {
                serde_json(&point);
            }
            Encoding::Yaml => {
                serde_yaml(&point);
            }
        },
        None => {
            trace!("Default encoding: json");
            serde_json(&point);
        }
    }
}
