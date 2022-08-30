use crate::serde::Point;

#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

pub fn encode(x: f64, y: f64) {
    trace!("yaml encode: x={:?} y={:?}", x, y);

    let point = Point { x: x, y: y };

    // Convert the Point to a YAML string.
    let serialized = serde_yaml::to_string(&point).unwrap();

    // Prints serialized = {"x":1,"y":2}
    info!("serialized = {}", serialized);
}

pub fn decode() {
    trace!("decode");

    let point = Point { x: 100., y: 200. };

    // Convert the Point to a YAML string.
    let serialized = serde_yaml::to_string(&point).unwrap();

    // Convert the JSON string back to a Point.
    let deserialized: Point = serde_yaml::from_str(&serialized).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    info!("deserialized YAML = {:?}", deserialized);
}
