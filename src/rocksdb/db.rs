#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use rocksdb::{IteratorMode, DB};
use simple_error::SimpleError;
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
struct ErrDbPathIsAFile {
    path: String,
}

impl ErrDbPathIsAFile {
    fn new(path: &str) -> ErrDbPathIsAFile {
        ErrDbPathIsAFile {
            path: path.to_string(),
        }
    }
}

impl fmt::Display for ErrDbPathIsAFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DbPath is a file: {}", self.path)
    }
}

impl Error for ErrDbPathIsAFile {
    fn description(&self) -> &str {
        &self.path
    }
}

// inverse of check_new_path
fn check_path(path: &str) -> Result<&Path, Box<dyn std::error::Error>> {
    match check_new_path(path) {
        Ok(p) => Err(Box::new(SimpleError::new(format!(
            "path doesn't exist: {}",
            p.display(),
        )))),
        Err(_) => Ok(Path::new(path)),
    }
}

fn check_new_path(path: &str) -> Result<&Path, Box<dyn std::error::Error>> {
    trace!("Checking path={}", path);
    let p = Path::new(path);
    if p.exists() {
        // artificial case to return a custom error type
        if p.is_file() {
            error!("Path is a file: {}", path);
            return Err(Box::new(ErrDbPathIsAFile::new(path)));
        }

        error!("Path already exists: {}", path);
        return Err(Box::new(SimpleError::new(format!(
            "Path already exists: {}",
            path
        ))));
    }
    Ok(p)
}

pub fn init(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Init path={}", path);
    let p = check_new_path(path)?;
    match DB::open_default(p) {
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

    let db = DB::open_default(check_path(path)?)?; //.unwrap();
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

    let db = DB::open_default(check_path(path)?)?; //.unwrap();
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
    let db = DB::open_default(check_path(path)?)?; //.unwrap();
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

    let db = DB::open_default(check_path(path)?)?; //.unwrap();
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
