#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use std::error::Error;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ErrBadDbPath {
    symlink: bool,
    path: String,
}

impl Error for ErrBadDbPath {}

impl ErrBadDbPath {
    pub fn file(path: &str) -> ErrBadDbPath {
        ErrBadDbPath {
            symlink: false,
            path: path.to_string(),
        }
    }
    pub fn symlink(path: &str) -> ErrBadDbPath {
        ErrBadDbPath {
            symlink: true,
            path: path.to_string(),
        }
    }
}

impl fmt::Display for ErrBadDbPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad path {:?}, symlink = {:?}", self.path, self.symlink)
    }
}

#[derive(Debug, Clone)]
pub struct ErrMissingIndex {
    cf_name: String,
}

impl Error for ErrMissingIndex {}

impl ErrMissingIndex {
    pub fn new(cf_name: String) -> ErrMissingIndex {
        ErrMissingIndex { cf_name }
    }
}

impl fmt::Display for ErrMissingIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing column family: {:?}", self.cf_name)
    }
}

#[derive(Debug, Clone)]
pub struct ErrNoCounters {
    cf_name: String,
}

impl ErrNoCounters {
    pub fn new(cf_name: String) -> ErrNoCounters {
        ErrNoCounters { cf_name }
    }
}
impl Error for ErrNoCounters {}

impl std::fmt::Display for ErrNoCounters {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Missing column family: {:?}", self.cf_name)
    }
}

#[derive(Debug, Clone)]
pub struct ErrBadIndex {
    cf_name: String,
    key: Vec<u8>,
}

impl ErrBadIndex {
    pub fn new(cf_name: String, key: &[u8]) -> ErrBadIndex {
        ErrBadIndex {
            cf_name,
            key: key.to_vec(),
        }
    }
}

impl fmt::Display for ErrBadIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad index: {:?}, key={:?}", self.cf_name, self.key)
    }
}

impl Error for ErrBadIndex {}
