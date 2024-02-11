#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use clap::{Args as clapArgs, Subcommand};

use std::path::PathBuf;
use std::vec::Vec;

//use duckdb::{Config, Connection, Error};

use crate::duckdb::db;
use crate::duckdb::Person;

#[derive(Debug, clapArgs)]
#[clap(args_conflicts_with_subcommands = false)]
pub struct Command {
    /// Path of the db file
    #[clap(long = "path", short = 'p')]
    pub path: PathBuf,

    #[clap(subcommand)]
    pub verb: Verb,
}

#[derive(Debug, Subcommand)]
pub enum Verb {
    Init(InitArgs),
    Create(CreateArgs),
    Read(ReadArgs),
    Update(UpdateArgs),
    Delete(DeleteArgs),
}

/// Initializes a database
#[derive(Debug, clapArgs)]
pub struct InitArgs {}

#[derive(Debug, clapArgs)]
pub struct CreateArgs {
    name: String,
    data: String,
}

#[derive(Debug, clapArgs)]
pub struct SelectArgs {
    #[clap(short = 'n')]
    name: Option<String>,

    #[clap(short = 'i')]
    id: Option<i32>,
}

#[derive(Debug, clapArgs)]
pub struct UpdateArgs {
    #[clap(short = 'i')]
    id: i32,

    #[clap(short = 'n')]
    name: Option<String>,

    #[clap(short = 'd')]
    data: Option<String>,
}

type ReadArgs = SelectArgs;
type DeleteArgs = SelectArgs;

pub fn go(cmd: &Command) {
    trace!("Running command: {:?}", cmd);

    let conn =
        match duckdb::Connection::open_with_flags(cmd.path.as_path(), duckdb::Config::default()) {
            Ok(c) => c,
            Err(err) => panic!("Got error: {:?}", err),
        };

    match &cmd.verb {
        Verb::Init(args) => {
            trace!("Init: {:?}", args);
            match db::initdb(&conn) {
                Ok(_) => info!("Done"),
                Err(e) => error!("Error: {:?}", e),
            }
        }
        Verb::Create(args) => {
            trace!("Create: {:?}, Context: {:?}", args, cmd);
            let person = Person {
                id: 0,
                name: args.name.clone(),
                data: Some(Vec::from(args.data.as_bytes())),
            };
            match db::insert_person(&conn, &person) {
                Ok(_) => info!("Inserted: {:?}", person),
                Err(e) => error!("Error: {:?}", e),
            }
        }
        Verb::Read(args) => {
            trace!("Read: {:?}", args);

            match &args.name {
                Some(name) => match db::read_person(&conn, "name=?", &name) {
                    Ok(cur) => {
                        for p in cur {
                            info!("{:?}", p);
                        }
                    }
                    Err(e) => error!("Error: {:?}", e),
                },
                None => {}
            }

            match args.id {
                Some(id) => match db::read_person(&conn, "id=?", &id) {
                    Ok(cur) => {
                        for p in cur {
                            info!("{:?}", p);
                        }
                    }
                    Err(e) => error!("Error: {:?}", e),
                },
                None => {}
            }
        }
        Verb::Update(args) => {
            trace!("Update: {:?}", args);

            let mut person = Person {
                id: args.id,
                ..Default::default()
            };

            person.name = match &args.name {
                Some(n) => n.clone(),
                None => "".to_string(),
            };

            person.data = match &args.data {
                Some(d) => Some(Vec::from(d.as_bytes())),
                None => None,
            };

            match db::update_person(&conn, &person) {
                Ok(p) => info!("Updated person {:?}", p),
                Err(e) => error!("Error: {:?}", e),
            }
        }
        Verb::Delete(args) => {
            trace!("Delete: {:?}", args);
            match &args.name {
                Some(name) => match db::delete_person(&conn, "name=?", &name) {
                    Ok(ret) => info!("deleted {:?}", ret),
                    Err(e) => error!("Error: {:?}", e),
                },
                None => {}
            }

            match args.id {
                Some(id) => match db::delete_person(&conn, "id=?", &id) {
                    Ok(ret) => info!("deleted {:?}", ret),
                    Err(e) => error!("Error: {:?}", e),
                },
                None => {}
            }
        }
    }

    match conn.close() {
        Ok(_) => {}
        Err(e) => error!("Error closing db: {:?}", e),
    }
}
