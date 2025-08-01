#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use crate::rocksdb::db::{Database, Entity, Transaction};
use crate::rocksdb::error::ErrMissingIndex;
use rocksdb::{Direction, IteratorMode};

use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Indexes<E: Entity> {
    fn indexes() -> Vec<Box<dyn Index<E>>>;
}

pub trait Index<E: Entity> {
    // Name of the column family that backs this index
    fn cf_name(&self) -> &'static str;

    // Returns the (key, value) for the index
    fn key_value(&self, e: &E) -> (Vec<u8>, Vec<u8>);

    fn prefixed_keys(&self) -> bool {
        false
    }

    // fn scan(
    //     &self,
    //     db: &Database,
    //     match_start: Vec<u8>, //&[u8],
    //     visitor: Box<dyn Visitor<E> + '_>,
    // ) -> Result<(), Box<dyn Error>> {
    //     let cf = db.cf_handle(self.cf_name()).unwrap();
    //     trace!("Found cf {:?} with match={:?}", self.cf_name(), match_start);
    //     let iter = db.iterator_cf(
    //         cf,
    //         IteratorMode::From(match_start.as_slice(), Direction::Forward),
    //     );
    //     for item in iter {
    //         let (k, v) = item.unwrap();
    //         trace!("Checking (k,v)={:?} | {:?}", String::from_utf8_lossy(&k), v);

    //         // The first bytes must match
    //         if k.len() < match_start.len() || match_start.to_owned() != k[0..match_start.len()] {
    //             break;
    //         }

    //         if v.len() == 0 {
    //             warn!(
    //                 "Bad entry: empty value for key {:?}",
    //                 String::from_utf8_lossy(&k),
    //             );
    //             break;
    //         }

    //         // Matches by prefix
    //         // 1. Index should implement method to return key from bytes
    //         // 2. Perform lookup and visit the entry
    //         info!(
    //             "Found match={:?}, (k,v)={:?} | {:?}",
    //             match_start,
    //             String::from_utf8_lossy(&k),
    //             v
    //         );

    //         let id = E::id_from(KeyCodec::decode_key(v.to_vec()));
    //         let stop: Result<bool, Box<dyn Error>> = match self.get(id)? {
    //             Some(obj) => Ok(!visitor.visit(obj)),
    //             None => {
    //                 let index_name = &self.cf_name().to_string().to_owned();
    //                 error!("Bad index!!! {:?}", ErrBadIndex::new(index_name, &v));
    //                 Err(Box::new(ErrBadIndex::new(index_name, &v)))
    //             }
    //         };
    //         if stop? {
    //             break;
    //         }
    //     }
    //     Ok(())
    // }

    fn update_entry(
        &self,
        db: &Database,
        txn: &mut Transaction,
        e: &E,
    ) -> Result<(), Box<dyn Error>> {
        match db.cf_handle(self.cf_name()) {
            Some(cf) => {
                let kv = self.key_value(e);
                if self.prefixed_keys() {
                    // get current timestamp in nanoseconds
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();
                    // compute the new key with timestamp
                    let new_key = format!("{}:{}", String::from_utf8_lossy(&kv.0), timestamp);
                    trace!(
                        "Append entry in index {:?}, (k,v) = ({:?},{:?})",
                        self.cf_name(),
                        new_key,
                        kv.1
                    );
                    txn.put_cf(cf, new_key, kv.1);
                } else {
                    trace!(
                        "Update entry in index {:?}, (k,v) = ({:?},{:?})",
                        self.cf_name(),
                        kv.0,
                        kv.1
                    );
                    txn.put_cf(cf, kv.0, kv.1);
                }
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
        db: &Database,
        txn: &mut Transaction,
        e: &E,
    ) -> Result<(), Box<dyn Error>> {
        match db.cf_handle(self.cf_name()) {
            Some(cf) => {
                if self.prefixed_keys() {
                    // if append only then delete all entries with the same prefix
                    let kv = self.key_value(e);
                    let match_key_string = format!("{}:", String::from_utf8_lossy(&kv.0));
                    let match_key = match_key_string.as_bytes();
                    trace!(
                        "Delete entry in index {:?}, key = {:?}",
                        self.cf_name(),
                        match_key,
                    );

                    let iter =
                        db.iterator_cf(cf, IteratorMode::From(match_key, Direction::Forward));
                    let mut target_keys = Vec::<Vec<u8>>::new();
                    for item in iter {
                        let (k, v) = item.unwrap();
                        if v.len() == 0 {
                            break;
                        }
                        trace!("For match={:?}, (k,v)={:?} | {:?}", match_key, k, v);
                        // The first bytes must match
                        if k.len() < match_key.len() {
                            break;
                        } else if match_key.to_owned() != k[0..match_key.len()] {
                            break;
                        }
                        target_keys.push(k.to_vec());
                    }
                    for k in target_keys {
                        trace!(
                            "Scheduled for deletion in index {:?}, key = {:?}",
                            self.cf_name(),
                            k,
                        );
                        txn.delete_cf(cf, k);
                    }
                    Ok(())
                } else {
                    let kv = self.key_value(e);
                    trace!(
                        "Delete entry in index {:?}, key = {:?}",
                        self.cf_name(),
                        kv.0,
                    );
                    txn.delete_cf(cf, kv.0);
                    Ok(())
                }
            }
            None => {
                trace!("Column family not found: {:?}", self.cf_name());
                Err(Box::new(ErrMissingIndex::new(self.cf_name().to_string())))
            }
        }
    }
}
