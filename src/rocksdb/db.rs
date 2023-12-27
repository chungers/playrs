#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use rocksdb::{Direction, IteratorMode, DB};
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
struct ErrBadDbPath {
    symlink: bool,
    path: String,
}

impl ErrBadDbPath {
    fn file(path: &str) -> ErrBadDbPath {
        ErrBadDbPath {
            symlink: false,
            path: path.to_string(),
        }
    }
    fn symlink(path: &str) -> ErrBadDbPath {
        ErrBadDbPath {
            symlink: true,
            path: path.to_string(),
        }
    }
    fn is_symlink(&self) -> bool {
        return self.symlink;
    }
}

impl fmt::Display for ErrBadDbPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Database path is not a directory: {:?} symlink={:?}",
            self.path,
            self.is_symlink()
        )
    }
}

impl Error for ErrBadDbPath {
    fn description(&self) -> &str {
        &self.path
    }
}

fn check_path(path: &str) -> Result<&Path, Box<dyn std::error::Error>> {
    let p = Path::new(path);
    if !p.exists() {
        return Ok(p); // rocksdb will create & init db at the path
    }
    if p.is_file() {
        error!("Path is a file: {}", path);
        return Err(Box::new(ErrBadDbPath::file(path)));
    }
    if p.is_symlink() {
        error!("Path is a file: {}", path);
        return Err(Box::new(ErrBadDbPath::symlink(path)));
    }
    Ok(p)
}

#[test]
fn test_check_path() {
    let p = Path::new("/bin");
    match check_path(p.to_str().unwrap()) {
        Ok(pp) => assert_eq!(pp, p),
        Err(_) => panic!("Valid directory /tmp"),
    }

    let p = Path::new("/bin/bash");
    match check_path(p.to_str().unwrap()) {
        Ok(_) => panic!("Shouldn't be ok with a file"),
        Err(_) => {}
    }
}

pub trait DbInfo {
    fn path(&self) -> &str;
}

pub trait VisitKV {
    fn visit(&self, _: &[u8], _: &[u8]);
}

pub fn init(info: &dyn DbInfo) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Init path={}", info.path());
    let p = check_path(info.path())?;
    match DB::open_default(p) {
        Ok(_) => {
            trace!("Db init at path = {}", info.path());
            Ok(())
        }
        Err(e) => {
            error!("Error init db at path = {}, error = {}", info.path(), e);
            Err(Box::new(e))
        }
    }
}

pub fn put(info: &dyn DbInfo, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Put path={}, key={}, value={}", info.path(), key, value);

    let db = DB::open_default(check_path(info.path())?)?;
    trace!("DB = {:?}", db);

    match db.put(key.as_bytes(), value.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("Error retrieving value for {}: {}", key, e);
            Err(Box::new(e))
        }
    }
}

pub fn get(
    info: &dyn DbInfo,
    key: &str,
    visitor: &dyn VisitKV,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    trace!("Get path={}, key={}", info.path(), key);

    let db = DB::open_default(check_path(info.path())?)?;
    trace!("DB = {:?}", db);

    match db.get(key.as_bytes()) {
        Ok(Some(v)) => {
            let result = String::from_utf8(v).unwrap();
            trace!("Finding '{}' returns '{}'", key, result);

            visitor.visit(key.as_bytes(), result.as_bytes());

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

pub fn delete(info: &dyn DbInfo, key: &str) -> Result<(), Box<dyn std::error::Error>> {
    trace!("Delete path={}, key={}", info.path(), key);
    let db = DB::open_default(check_path(info.path())?)?;
    trace!("DB = {:?}", db);
    match db.delete(key.as_bytes()) {
        Ok(()) => Ok(()),
        Err(e) => {
            error!("Error retrieving value for {}: {}", key, e);
            return Err(Box::new(e));
        }
    }
}

pub fn list(
    info: &dyn DbInfo,
    key: &str,
    visitor: &dyn VisitKV,
) -> Result<(), Box<dyn std::error::Error>> {
    trace!("List path={}, key={}", info.path(), key);
    let db = DB::open_default(check_path(info.path())?)?;
    trace!("DB = {:?}", db);
    let iter = db.iterator(IteratorMode::From(key.as_bytes(), Direction::Forward));
    for item in iter {
        let (k, v) = item.unwrap();
        visitor.visit(&k, &v);
    }
    Ok(())
}
