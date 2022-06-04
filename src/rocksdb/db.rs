#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use rocksdb::{IteratorMode, DB};
use simple_error::SimpleError;
use std::path::Path;

fn open(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Opening database at {}", path);
    let db = DB::open_default(path).unwrap();
    info!("DB = {:?}", db);
    Ok(())
}

pub fn init(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Init path={}", path);
    if Path::new(path).exists() {
        error!("Path already exists: {}", path);
        return Err(Box::new(SimpleError::new(format!(
            "Path already exists: {}",
            path
        ))));
    }
    match DB::open_default(path) {
        Ok(_) => {
            info!("Db init at path = {}", path);
            Ok(())
        }
        Err(e) => {
            error!("Error init db at path = {}, error = {}", path, e);
            Err(Box::new(e))
        }
    }
}

pub fn put(path: &str, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Put path={}, key={}, value={}", path, key, value);

    let db = DB::open_default(path).unwrap();
    info!("DB = {:?}", db);

    match db.put(key.as_bytes(), value.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("Error retrieving value for {}: {}", key, e);
            Err(Box::new(e))
        }
    }
}

pub fn get(path: &str, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    trace!("Get path={}, key={}", path, key);

    let db = DB::open_default(path).unwrap();
    info!("DB = {:?}", db);

    match db.get(key.as_bytes()) {
        Ok(Some(v)) => {
            let result = String::from_utf8(v).unwrap();
            trace!("Finding '{}' returns '{}'", key, result);
            Ok(Some(result))
        }
        Ok(None) => {
            trace!("Finding '{}' returns None", key);
            Ok(None)
        }
        Err(e) => {
            error!("Error retrieving value for {}: {}", key, e);
            Err(Box::new(e))
        }
    }
}

pub fn delete(path: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Put path={}, key={}", path, key);
    let db = DB::open_default(path).unwrap();
    info!("DB = {:?}", db);
    match db.delete(key.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("Error retrieving value for {}: {}", key, e);
            return Err(Box::new(e));
        }
    }
}

pub fn list(path: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Put path={}, key={}", path, key);

    let db = DB::open_default(path).unwrap();
    info!("DB = {:?}", db);
    let iter = db.iterator(IteratorMode::Start); // Always iterates forward
    for (k, v) in iter {
        println!(
            "{} {}",
            String::from_utf8(k.to_vec()).unwrap(),
            String::from_utf8(v.to_vec()).unwrap()
        );
    }
    Ok(())
}
