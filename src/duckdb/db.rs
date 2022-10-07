#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use duckdb::{params, Connection, Result};

use arrow::record_batch::RecordBatch;
//use arrow::util::pretty::print_batches;

use simple_error::SimpleError;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::duckdb::Person;

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

pub fn example() -> Result<(), Box<dyn std::error::Error>> {
    trace!("example");

    let conn = Connection::open_in_memory()?;

    conn.execute_batch(
        r"CREATE SEQUENCE seq;
          CREATE TABLE person (
                  id              INTEGER PRIMARY KEY DEFAULT NEXTVAL('seq'),
                  name            TEXT NOT NULL,
                  data            BLOB
                  );
        ",
    )?;

    let me = Person {
        id: 0,
        name: "Steven".to_string(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, data) VALUES (?, ?)",
        params![me.name, me.data],
    )?;

    // query table by rows
    let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            data: row.get(2)?,
        })
    })?;

    for person in person_iter {
        println!("Found person {:?}", person.unwrap());
    }

    /*
        // query table by arrow
        let rbs: Vec<RecordBatch> = stmt.query_arrow([])?.collect();
        print_batches(&rbs);
    */
    Ok(())
}
