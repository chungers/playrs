#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;
use crate::rocksdb::db::{Entity, HasKey};

use std::convert::TryInto;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Counter {
    key: String,
    value: u64,
}

impl Counter {
    fn new(key: &str) -> Counter {
        Counter {
            key: key.to_string(),
            value: 0u64,
        }
    }

    pub fn set(&mut self, v: u64) {
        self.value = v;
    }
    pub fn get(&self) -> u64 {
        self.value
    }
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

pub(crate) struct Counters<'a> {
    db: &'a db::Database,
    column_family: String,
}

impl Counters<'_> {
    pub fn new<'a>(db: &'a db::Database, cf: &'a str) -> Counters<'a> {
        Counters {
            db: db,
            column_family: cf.to_string(),
        }
    }

    // Creates a new counter by key if not found.
    pub fn get(&self, key: &str) -> Result<Counter, Box<dyn Error>> {
        let cf = self.db.cf_handle(self.column_family.as_str()).unwrap();
        match self.db.get_cf(cf, key) {
            Ok(Some(bytes)) => Counter::from_bytes(key.as_bytes(), &bytes),
            Ok(None) => Ok(Counter::new(key)),
            Err(e) => Err(Box::new(e)),
        }
    }

    // Update the counter in db
    pub fn update(
        &mut self,
        txn: &mut db::Transaction,
        counter: &Counter,
    ) -> Result<(), Box<dyn Error>> {
        match self.db.cf_handle(self.column_family.as_str()) {
            Some(cf) => {
                txn.put_cf(cf, counter.id().as_bytes(), counter.as_bytes());
                Ok(())
            }
            None => Err(Box::new(ErrNoCounters {
                cf_name: self.column_family.to_string(),
            })),
        }
    }
}

#[derive(Debug, Clone)]
struct ErrNoCounters {
    cf_name: String,
}

impl Error for ErrNoCounters {}

impl std::fmt::Display for ErrNoCounters {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing column family: {:?}", self.cf_name)
    }
}

#[test]
fn test_using_counters() {
    println!("Counter ***********************************");

    let c1 = Counter {
        key: "c1".into(),
        value: 2_u64.pow(8),
    };

    assert_eq!(Counter::TYPE, "Counter");

    println!("Counter = {:?}, bytes = {:?}", c1, c1.as_bytes());
    println!("Id = {:?}", c1.id());

    let mut c11 = Counter::new("c1");
    c11.set(2_u64.pow(8));
    assert_eq!(c1, c11);
    assert_eq!(c1.as_bytes(), c11.as_bytes());
    assert_eq!(c1.id(), c11.id());

    let c2 = Counter::from_bytes(c1.key.as_bytes(), &c1.as_bytes()).unwrap();
    println!("C2 = {:?}", c2);
    assert_eq!(c1, c2);
}
