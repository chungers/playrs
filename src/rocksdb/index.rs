#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;

use std::error::Error;
use std::fmt;

pub trait Indexes<E: db::Entity> {
    fn indexes() -> Vec<Box<dyn Index<E>>>;
}

// TODO - fix this. Queries can depend on the Entity...
pub trait Queries<E: db::Entity> {}

#[derive(Debug, Clone)]
struct ErrMissingIndex {
    cf_name: String,
}

impl Error for ErrMissingIndex {}

impl fmt::Display for ErrMissingIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing column family: {:?}", self.cf_name)
    }
}

pub trait Index<E: db::Entity + std::fmt::Debug> {
    // Name of the column family
    fn cf_name(&self) -> &'static str;

    // Default implementation is to store the bytes of the entity
    fn key_value(&self, e: &E) -> (Vec<u8>, Vec<u8>) {
        return (e.key(), e.as_bytes());
    }

    fn update_entry(
        &self,
        db: &db::Database,
        txn: &mut db::Transaction,
        e: &E,
    ) -> Result<(), Box<dyn Error>> {
        match db.cf_handle(self.cf_name()) {
            Some(cf) => {
                let kv = self.key_value(e);

                trace!(
                    "update entry in index {:?}, (k,v) = ({:?},{:?})",
                    self.cf_name(),
                    kv.0,
                    kv.1
                );
                txn.put_cf(cf, kv.0, kv.1);
                Ok(())
            }
            None => {
                trace!("Column family not found: {:?}", self.cf_name());
                Err(Box::new(ErrMissingIndex {
                    cf_name: self.cf_name().to_string(),
                }))
            }
        }
    }
    fn delete_entry(&self, e: &E) -> Result<(), Box<dyn Error>> {
        trace!(
            "delete entry in index {:?}, entry = {:?}",
            self.cf_name(),
            e
        );
        Ok(())
    }
}
