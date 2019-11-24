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
extern crate eloquentlog_console_api;

mod authentication;
mod message;
mod error;
mod registration;
mod top;
mod user;
mod password_reset;
mod password_reset_request;

use std::panic::{self, AssertUnwindSafe};
use regex::Regex;

use diesel::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use chrono::{Utc, TimeZone};
use fnv::FnvHashMap;
use parking_lot::Mutex;
use rocket::local::Client;
use rocket_slog::SlogFairing;
use uuid::Uuid;

use eloquentlog_console_api::server;
use eloquentlog_console_api::db;
use eloquentlog_console_api::mq;
use eloquentlog_console_api::ss;
use eloquentlog_console_api::config;
use eloquentlog_console_api::logger;
use eloquentlog_console_api::model;

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
    static ref DB_POOL_HOLDER: db::DbPoolHolder = {
        db::init_pool_holder(
            &CONFIG.database_url,
            CONFIG.database_max_pool_size,
        )
    };
    static ref MQ_POOL_HOLDER: mq::MqPoolHolder = {
        mq::init_pool_holder(
            &CONFIG.message_queue_url,
            CONFIG.message_queue_max_pool_size,
        )
    };
    static ref SS_POOL_HOLDER: ss::SsPoolHolder = {
        ss::init_pool_holder(
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
where T: FnOnce(&Client, &mut Connection, &config::Config, &logger::Logger) -> ()
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

    let logger = logger::get_logger(&CONFIG);
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
pub fn get_db_conn(holder: &db::DbPoolHolder) -> db::DbConn {
    holder.get().map(db::DbConn).expect("databane connection")
}

pub fn get_mq_conn(holder: &mq::MqPoolHolder) -> mq::MqConn {
    holder
        .get()
        .map(mq::MqConn)
        .expect("message queue connection")
}

pub fn get_ss_conn(holder: &ss::SsPoolHolder) -> ss::SsConn {
    holder
        .get()
        .map(ss::SsConn)
        .expect("session store connection")
}

// test data fixtures

type UserFixture = FnvHashMap<&'static str, model::user::User>;

lazy_static! {
    pub static ref USERS: UserFixture = fnvhashmap! {
        "oswald" => model::user::User {
            id: 1,
            uuid: Uuid::new_v4(),
            name: Some("Oswald".to_string()),
            username: "oswald".to_string(),
            email: "oswald@example.org".to_string(),
            password: b"Pa$$w0rd".to_vec(),
            state: model::user::UserState::Active,
            reset_password_state: model::user::UserResetPasswordState::Never,
            reset_password_token: None,
            reset_password_token_expires_at: None,
            reset_password_token_granted_at: None,
            created_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
            updated_at: Utc.ymd(2019, 7, 7).and_hms(7, 20, 15).naive_utc(),
        }
    };
}

// test utils

fn load_user(
    mut user: model::user::User,
    db_conn: &PgConnection,
) -> model::user::User
{
    user.change_password(&make_raw_password(&user));

    let result: Result<model::user::User, diesel::result::Error> = db_conn
        .build_transaction()
        .run::<model::user::User, diesel::result::Error, _>(|| {
            let q = diesel::insert_into(model::user::users::table).values(user);
            user = q
                .get_result::<model::user::User>(db_conn)
                .unwrap_or_else(|e| panic!("e: {}", e));

            let q = diesel::insert_into(model::user_email::user_emails::table)
                .values((
                    model::user_email::user_emails::user_id.eq(&user.id),
                    Some(model::user_email::user_emails::email.eq(&user.email)),
                    model::user_email::user_emails::role
                        .eq(model::user_email::UserEmailRole::Primary),
                    model::user_email::user_emails::identification_state.eq(
                        model::user_email::UserEmailIdentificationState::Done,
                    ),
                ));
            let _ = q
                .get_result::<model::user_email::UserEmail>(db_conn)
                .unwrap_or_else(|e| panic!("e: {}", e));

            Ok(user)
        });
    result.unwrap()
}

/// Creates raw password string.
///
/// It works only in test because USERS has `password` as dummy `Vec<u8>`.
fn make_raw_password(user: &model::user::User) -> String {
    user.password.iter().map(|c| *c as char).collect()
}
