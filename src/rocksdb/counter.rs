#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;

use std::convert::TryInto;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Counter {
    key: String,
    value: u64,
}

impl db::HasKey<String> for Counter {
    fn key(&self) -> Option<String> {
        Some(self.key.to_string())
    }
}

impl db::Entity for Counter {
    const TYPE: &'static str = "Counter";
    fn as_bytes(&self) -> Vec<u8> {
        self.value.to_le_bytes().to_vec()
    }
    fn from_bytes(key: &[u8], bytes: &[u8]) -> Result<Counter, Box<dyn Error>> {
        let be = bytes.try_into().unwrap();
        let val = u64::from_le_bytes(be);
        Ok(Counter {
            key: String::from_utf8(key.to_vec()).unwrap().into(),
            value: val,
        })
    }
}

#[test]
fn test_using_counters() {
    println!("Counter");

    use db::Entity;
    let c1 = Counter {
        key: "c1".into(),
        value: 2_u64.pow(8),
    };
    println!("Counter = {:?}, bytes = {:?}", c1, c1.as_bytes());

    let c2 = Counter::from_bytes(c1.key.as_bytes(), &c1.as_bytes()).unwrap();
    println!("C2 = {:?}", c2);

    assert_eq!(c1, c2);
}
