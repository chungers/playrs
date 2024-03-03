#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db;
use crate::rocksdb::error::ErrMissingIndex;

use std::error::Error;

pub trait Indexes<E: db::Entity> {
    fn indexes() -> Vec<Box<dyn Index<E>>>;
}

// TODO - fix this. Queries can depend on the Entity...
pub trait Queries<E: db::Entity> {}

pub trait Index<E: db::Entity> {
    // Name of the column family that backs this index
    fn cf_name(&self) -> &'static str;

    // Returns the (key, value) for the index
    fn key_value(&self, e: &E) -> (Vec<u8>, Vec<u8>);

    // TODO -
    // Index keys can change based on the fields changed.
    // If we had index on obj.foo and obj.bar and now we have
    // obj.foo' and obj.bar', we need to 1) remove the index
    // keyed by (obj.foo, obj.bar) and add (obj.foo', obj.bar').
    // This means we need to have a copy of the old value of e.
    // Under the covers, before updating the value index, which
    // stores (id, value), we have to read the old value and then
    // remove the index entry at (value.foo, value.bar) and the
    // add the index entry at (value.foo', value.bar').
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
                    "Update entry in index {:?}, (k,v) = ({:?},{:?})",
                    self.cf_name(),
                    kv.0,
                    kv.1
                );
                txn.put_cf(cf, kv.0, kv.1);
                Ok(())
            }
            None => {
                trace!("Column family not found: {:?}", self.cf_name());
                Err(Box::new(ErrMissingIndex::new(self.cf_name().to_string())))
            }
        }
    }
    fn delete_entry(
        &self,
        db: &db::Database,
        txn: &mut db::Transaction,
        e: &E,
    ) -> Result<(), Box<dyn Error>> {
        match db.cf_handle(self.cf_name()) {
            Some(cf) => {
                let kv = self.key_value(e);
                trace!(
                    "Delete entry in index {:?}, (k,v) = ({:?},{:?})",
                    self.cf_name(),
                    kv.0,
                    kv.1,
                );
                txn.put_cf(cf, kv.0, vec![]);
                Ok(())
            }
            None => {
                trace!("Column family not found: {:?}", self.cf_name());
                Err(Box::new(ErrMissingIndex::new(self.cf_name().to_string())))
            }
        }
    }
}
