#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}

impl Default for Person {
    fn default() -> Person {
        Person {
            id: -1,
            name: "".to_string(),
            data: None,
        }
    }
}

pub mod command;
pub mod db;
