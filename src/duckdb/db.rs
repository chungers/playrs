#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use duckdb::{params, Connection, Result, ToSql};

//use std::error::Error;
//use std::fmt;
use std::vec::Vec;

use crate::duckdb::Person;

pub fn read_person<KT: ToSql>(
    conn: &Connection,
    whereq: &str,
    val: &KT,
) -> Result<Vec<Box<Person>>, Box<dyn std::error::Error>> {
    let sql: String = "SELECT id, name, data FROM person WHERE ".to_string() + whereq;
    let mut stmt = conn.prepare(&sql)?;
    let person_iter = stmt.query_map([val], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            data: row.get(2)?,
        })
    })?;

    let mut v: Vec<Box<Person>> = Vec::new();
    for person in person_iter {
        v.push(Box::new(person.unwrap()));
    }

    trace!("got vec: {:?}", v);
    Ok(v)
}

pub fn delete_person<KT: ToSql>(
    conn: &Connection,
    whereq: &str,
    val: &KT,
) -> Result<usize, Box<dyn std::error::Error>> {
    let sql: String = "DELETE FROM person WHERE ".to_string() + whereq;
    let mut stmt = conn.prepare(&sql)?;
    match stmt.execute([val]) {
        Ok(u) => {
            trace!("ret={:?}", u);
            Ok(u)
        }
        Err(e) => Err(Box::new(e)),
    }
}

pub fn insert_person<'a>(
    conn: &Connection,
    person: &'a Person,
) -> Result<&'a Person, Box<dyn std::error::Error>> {
    trace!("inserting with conn={:?}, person={:?}", conn, person);

    conn.execute(
        r#"INSERT INTO person (name, data) VALUES (?, ?)"#,
        params![person.name, person.data],
    )?;

    Ok(person)
}

pub fn update_person<'a>(
    conn: &Connection,
    person: &'a Person,
) -> Result<&'a Person, Box<dyn std::error::Error>> {
    trace!("updating with conn={:?}, person={:?}", conn, person);

    conn.execute(
        r#"UPDATE person SET name=? , data=? WHERE id=?"#,
        params![person.name, person.data, person.id],
    )?;

    Ok(person)
}

pub fn initdb(conn: &Connection) -> Result<()> {
    trace!("initdb with conn={:?}", conn);
    conn.execute_batch(
        r#"CREATE SEQUENCE IF NOT EXISTS seq;
                  CREATE TABLE IF NOT EXISTS person (
                      id              INTEGER PRIMARY KEY DEFAULT NEXTVAL('seq'),
                      name            TEXT NOT NULL,
                      data            BLOB
                  );"#,
    )
}
