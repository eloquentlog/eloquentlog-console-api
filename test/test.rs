extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate diesel;
extern crate dotenv;
extern crate fourche;
extern crate fnv;
extern crate parking_lot;
extern crate redis;
extern crate rocket;
extern crate rocket_slog;
extern crate serde_json;
extern crate uuid;

#[macro_use]
extern crate eloquentlog_backend_api;

mod authentication;
mod message;
mod error;
mod registration;
mod top;
mod activation;

use std::panic::{self, AssertUnwindSafe};
use regex::Regex;

use diesel::prelude::*;
use diesel::PgConnection;
use dotenv::dotenv;
use chrono::{Utc, TimeZone};
use fnv::FnvHashMap;
use parking_lot::Mutex;
use rocket::local::Client;
use rocket_slog::SlogFairing;
use uuid::Uuid;

use eloquentlog_backend_api::server;
use eloquentlog_backend_api::db::{
    DbConn, DbPoolHolder, init_pool_holder as init_db_pool_holder,
};
use eloquentlog_backend_api::mq::{
    MqConn, MqPoolHolder, init_pool_holder as init_mq_pool_holder,
};
use eloquentlog_backend_api::ss::{
    SsConn, SsPoolHolder, init_pool_holder as init_ss_pool_holder,
};
use eloquentlog_backend_api::config;
use eloquentlog_backend_api::logger::{Logger, get_logger};
use eloquentlog_backend_api::model::user;

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
    static ref CONFIG: config::Config = {
        dotenv().ok();
        config::Config::from("testing").unwrap()
    };
    static ref DB_POOL_HOLDER: DbPoolHolder = {
        init_db_pool_holder(&CONFIG.database_url, CONFIG.database_max_pool_size)
    };
    static ref MQ_POOL_HOLDER: MqPoolHolder = {
        init_mq_pool_holder(
            &CONFIG.message_queue_url,
            CONFIG.message_queue_max_pool_size,
        )
    };
    static ref SS_POOL_HOLDER: SsPoolHolder = {
        init_ss_pool_holder(
            &CONFIG.session_store_url,
            CONFIG.session_store_max_pool_size,
        )
    };
}

pub struct Connection<'a> {
    db: &'a PgConnection,
    mq: &'a mut redis::Connection,
    ss: &'a mut redis::Connection,
}

/// Formats JSON text as one line
pub fn minify(s: String) -> String {
    RE.replace_all(&s, "$1").to_string()
}

/// A test runner for integration tests
pub fn run_test<T>(test: T)
where T: FnOnce(&Client, &mut Connection, &config::Config, &Logger) -> ()
        + panic::UnwindSafe {
    let _lock = DB_LOCK.lock();

    // Use same connection pools across tests
    let db_conn = get_db_conn(&DB_POOL_HOLDER.clone());
    let mut mq_conn = get_mq_conn(&MQ_POOL_HOLDER.clone());
    let mut ss_conn = get_ss_conn(&SS_POOL_HOLDER.clone());

    let mut conn = Connection {
        db: &db_conn,
        mq: &mut mq_conn,
        ss: &mut ss_conn,
    };

    let logger = get_logger(&CONFIG);
    setup(&mut conn);

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let server = server()
            .attach(SlogFairing::new(logger.clone()))
            .manage(DB_POOL_HOLDER.clone())
            .manage(MQ_POOL_HOLDER.clone())
            .manage(SS_POOL_HOLDER.clone())
            .manage(CONFIG.clone());
        let client = Client::new(server).unwrap();

        test(&client, &mut conn, &CONFIG, &logger)
    }));
    assert!(result.is_ok());

    teardown(&mut conn);
}

fn setup(conn: &mut Connection) {
    clean(conn);
}

fn teardown(conn: &mut Connection) {
    clean(conn);
}

fn clean(conn: &mut Connection) {
    redis::cmd("FLUSHDB").execute(conn.mq);
    redis::cmd("FLUSHDB").execute(conn.ss);

    // Postgres >= 9.5
    let q = r#"
DO $func$
BEGIN
  EXECUTE (
    SELECT
      'TRUNCATE TABLE ' || string_agg(oid::regclass::text, ', ') || ' CASCADE'
    FROM
      pg_class
    WHERE
      relkind = 'r' AND
      relnamespace = 'public'::regnamespace AND
      oid::regclass::text != '__diesel_schema_migrations'
  );
END $func$;
            "#;
    let _ = diesel::sql_query(q)
        .execute(conn.db)
        .expect("Failed to delete");
}

// TODO: move these function into {db,mq,ss}.rs.
pub fn get_db_conn(holder: &DbPoolHolder) -> DbConn {
    holder.get().map(DbConn).expect("databane connection")
}

pub fn get_mq_conn(holder: &MqPoolHolder) -> MqConn {
    holder.get().map(MqConn).expect("message queue connection")
}

pub fn get_ss_conn(holder: &SsPoolHolder) -> SsConn {
    holder.get().map(SsConn).expect("session store connection")
}

// test data fixtures

type UserFixture = FnvHashMap<&'static str, user::User>;

lazy_static! {
    pub static ref USERS: UserFixture = fnvhashmap! {
        "oswald" => user::User {
            id: 1,
            uuid: Uuid::new_v4(),
            name: Some("Oswald".to_string()),
            username: "oswald".to_string(),
            email: "oswald@example.org".to_string(),
            password: b"Pa$$w0rd".to_vec(),
            state: user::UserState::Active,
            reset_password_state: user::UserResetPasswordState::Never,
            reset_password_token: None,
            reset_password_token_expires_at: None,
            reset_password_token_granted_at: None,
            created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
        }
    };
}

// test utils

fn load_user(mut user: user::User, db_conn: &PgConnection) -> user::User {
    user.change_password(&make_raw_password(&user));

    diesel::insert_into(user::users::table)
        .values(user)
        .get_result::<user::User>(db_conn)
        .unwrap_or_else(|e| panic!("Error at inserting: {}", e))
}

/// Creates raw password string.
///
/// It works only in test because USERS has `password` as dummy `Vec<u8>`.
fn make_raw_password(user: &user::User) -> String {
    user.password.iter().map(|c| *c as char).collect()
}
