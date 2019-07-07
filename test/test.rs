extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate diesel;
extern crate dotenv;
extern crate fourche;
extern crate parking_lot;
extern crate redis;
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
use eloquentlog_backend_api::db::{DbConn, DbPool, init_pool as init_db_pool};
use eloquentlog_backend_api::mq::{MqConn, MqPool, init_pool as init_mq_pool};
use eloquentlog_backend_api::config::Config;
use eloquentlog_backend_api::logger::{Logger, get_logger};

// NOTE:
// For now, run tests sequencially :'(
// The usage of transactions for the same connection between tests and
// client (server) might fix this issue, but we use connection pool.
// Find another way.
lazy_static! {
    static ref DB_LOCK: Mutex<()> = Mutex::new(());
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"\n\s{2}|\n|(:)\s").unwrap();
    static ref CONFIG: Config = {
        dotenv().ok();
        Config::from("testing").unwrap()
    };
    static ref DB_POOL: DbPool =
        { init_db_pool(&CONFIG.database_url, CONFIG.database_max_pool_size) };
    static ref MQ_POOL: MqPool =
        { init_mq_pool(&CONFIG.queue_url, CONFIG.queue_max_pool_size) };
}

/// Formats JSON text as one line
pub fn minify(s: String) -> String {
    RE.replace_all(&s, "$1").to_string()
}

/// A test runner for integration tests
pub fn run_test<T>(test: T)
where T: FnOnce(
            Client,
            &PgConnection,
            &redis::Connection,
            &Config,
            &Logger,
        ) -> ()
        + panic::UnwindSafe {
    let _lock = DB_LOCK.lock();

    // Use same connection pools between test and client
    let db_conn = get_db_conn(&DB_POOL);
    let mq_conn = get_mq_conn(&MQ_POOL);

    let logger = get_logger(&CONFIG);
    setup(&db_conn, &mq_conn);

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let server = server(&CONFIG)
            .manage(DB_POOL.clone())
            .manage(MQ_POOL.clone());
        let client = Client::new(server).unwrap();

        test(client, &db_conn, &mq_conn, &CONFIG, &logger)
    }));
    assert!(result.is_ok());

    teardown(&db_conn, &mq_conn);
}

fn setup(db_conn: &PgConnection, mq_conn: &redis::Connection) {
    clean(db_conn, mq_conn);
}

fn teardown(db_conn: &PgConnection, mq_conn: &redis::Connection) {
    clean(db_conn, mq_conn);
}

fn clean(db_conn: &PgConnection, _: &redis::Connection) {
    let _: std::result::Result<(), diesel::result::Error> = db_conn
        .build_transaction()
        .serializable()
        .deferrable()
        .read_write()
        .run(|| {
            // TODO: back to TRUNCATE with ALTER TABLE for REFERENCES
            for table in ["messages", "user_emails", "users"].iter() {
                let _ = diesel::sql_query(format!("DELETE FROM {};", table))
                    .execute(db_conn)
                    .expect("Failed to delete");

                let _ = diesel::sql_query(format!(
                    "ALTER SEQUENCE {}_id_seq RESTART WITH 1;",
                    table
                ))
                .execute(db_conn)
                .expect("Failed to reset sequence");
            }
            Ok(())
        });
}

pub fn get_db_conn(connection_pool: &DbPool) -> DbConn {
    match connection_pool.get() {
        Ok(conn) => DbConn(conn),
        Err(e) => panic!("err: {}", e),
    }
}

pub fn get_mq_conn(connection_pool: &MqPool) -> MqConn {
    match connection_pool.get() {
        Ok(conn) => MqConn(conn),
        Err(e) => panic!("err: {}", e),
    }
}
