extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate diesel;
extern crate dotenv;
extern crate parking_lot;
extern crate rocket;

extern crate eloquentlog_backend_api;

mod login;
mod message;
mod error;
mod user;
mod top;

use std::panic::{self, AssertUnwindSafe};
use regex::Regex;

use diesel::prelude::*;
use diesel::PgConnection;
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
where T: FnOnce(Client, &PgConnection) -> () + panic::UnwindSafe {
    // NOTE:
    // For now, run tests sequencially :'(
    // The usage of transactions for the same connection between tests and
    // client (server) might fix this issue, but we use connection pool.
    // Find another way.
    lazy_static! {
        static ref DB_LOCK: Mutex<()> = Mutex::new(());
    }
    let _lock = DB_LOCK.lock();

    dotenv::dotenv().ok();

    // Use same connection pool between test and client
    let connection_pool = get_pool();
    let conn = get_conn(&connection_pool);
    setup(&conn);

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let client = Client::new(server().manage(connection_pool)).unwrap();

        test(client, &conn)
    }));
    assert!(result.is_ok());

    teardown(&conn);
}

fn setup(conn: &PgConnection) {
    truncate_messages(conn);
}

fn teardown(conn: &PgConnection) {
    truncate_messages(conn);
}

pub fn truncate_messages(conn: &PgConnection) {
    let _: std::result::Result<(), diesel::result::Error> = conn
        .build_transaction()
        .serializable()
        .deferrable()
        .read_write()
        .run(|| {
            let _ = diesel::sql_query("TRUNCATE TABLE messages;")
                .execute(conn)
                .expect("Failed to truncate");

            let _ = diesel::sql_query(
                "ALTER SEQUENCE messages_id_seq RESTART WITH 1;",
            )
            .execute(conn)
            .expect("Failed to reset sequence");
            Ok(())
        });
}

pub fn get_pool() -> Pool {
    let c = Config::from("testing").unwrap();
    init_pool(&c.database_url)
}

pub fn get_conn(connection_pool: &Pool) -> DbConn {
    match connection_pool.get() {
        Ok(conn) => DbConn(conn),
        Err(e) => panic!("err: {}", e),
    }
}
