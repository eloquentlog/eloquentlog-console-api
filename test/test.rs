extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate diesel;
extern crate dotenv;
extern crate parking_lot;
extern crate rocket;

extern crate eloquentlog_backend_api;

mod authentication;
mod message;
mod error;
mod registration;
mod top;

use std::panic::{self, AssertUnwindSafe};
use regex::Regex;

use diesel::prelude::*;
use diesel::PgConnection;
use dotenv::dotenv;
use parking_lot::Mutex;
use rocket::local::Client;

use eloquentlog_backend_api::server;
use eloquentlog_backend_api::db::{DbConn, Pool, init_pool};
use eloquentlog_backend_api::config::Config;

/// Formats JSON text as one line
pub fn minify(s: String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\n\s{2}|\n|(:)\s").unwrap();
    }
    RE.replace_all(&s, "$1").to_string()
}

/// A test runner for integration tests
pub fn run_test<T>(test: T)
where T: FnOnce(Client, &PgConnection, &Config) -> () + panic::UnwindSafe {
    // NOTE:
    // For now, run tests sequencially :'(
    // The usage of transactions for the same connection between tests and
    // client (server) might fix this issue, but we use connection pool.
    // Find another way.
    lazy_static! {
        static ref DB_LOCK: Mutex<()> = Mutex::new(());
    }
    let _lock = DB_LOCK.lock();

    dotenv().ok();
    let config = Config::from("testing").unwrap();

    // Use same connection pool between test and client
    let connection_pool = get_pool(&config);
    let conn = get_conn(&connection_pool);
    setup(&conn);

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let server = server(&config).manage(connection_pool);
        let client = Client::new(server).unwrap();

        test(client, &conn, &config)
    }));
    assert!(result.is_ok());

    teardown(&conn);
}

fn setup(conn: &PgConnection) {
    clean(conn);
}

fn teardown(conn: &PgConnection) {
    clean(conn);
}

fn clean(conn: &PgConnection) {
    let _: std::result::Result<(), diesel::result::Error> = conn
        .build_transaction()
        .serializable()
        .deferrable()
        .read_write()
        .run(|| {
            // TODO: back to TRUNCATE with ALTER TABLE for REFERENCES
            for table in ["messages", "user_emails", "users"].iter() {
                let _ = diesel::sql_query(format!("DELETE FROM {};", table))
                    .execute(conn)
                    .expect("Failed to delete");

                let _ = diesel::sql_query(format!(
                    "ALTER SEQUENCE {}_id_seq RESTART WITH 1;",
                    table
                ))
                .execute(conn)
                .expect("Failed to reset sequence");
            }
            Ok(())
        });
}

pub fn get_pool(config: &Config) -> Pool {
    init_pool(&config.database_url)
}

pub fn get_conn(connection_pool: &Pool) -> DbConn {
    match connection_pool.get() {
        Ok(conn) => DbConn(conn),
        Err(e) => panic!("err: {}", e),
    }
}
