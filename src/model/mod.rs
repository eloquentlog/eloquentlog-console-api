//! Model entities and SQL types.
//!
//! SQL types are imported publicly in each model entities.

// sql types
mod log_level;
mod log_format;
mod user_email_verification_state;
mod user_email_role;
mod user_state;
mod user_reset_password_state;

// non-persistent (deciduous) entities
pub mod token;

// models
pub mod message;
pub mod user;
pub mod user_email;

#[cfg(test)]
pub mod test {
    use std::panic::{self, AssertUnwindSafe};

    use diesel::{PgConnection, prelude::*};
    use dotenv::dotenv;

    use config::Config;
    use db::{DbPoolHolder, init_pool_holder};
    use logger::{Logger, get_logger};

    lazy_static! {
        static ref CONFIG: Config = {
            dotenv().ok();
            Config::from("testing").unwrap()
        };
        static ref DB_POOL_HOLDER: DbPoolHolder = {
            init_pool_holder(
                &CONFIG.database_url,
                CONFIG.database_max_pool_size,
            )
        };
    }

    /// A test runner
    pub fn run<T>(test: T)
    where T: FnOnce(&PgConnection, &Config, &Logger) -> () + panic::UnwindSafe
    {
        let conn = DB_POOL_HOLDER.get().expect("database connection");
        let logger = get_logger(&CONFIG);

        let _: std::result::Result<(), diesel::result::Error> =
            conn.build_transaction().read_write().run(|| {
                setup(&conn);

                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    test(&conn, &CONFIG, &logger)
                }));

                teardown(&conn);

                assert!(result.is_ok());
                Ok(())
            });
    }

    fn setup(conn: &PgConnection) {
        clean(conn);
    }

    fn teardown(conn: &PgConnection) {
        clean(conn);
    }

    pub fn clean(conn: &PgConnection) {
        let tables = ["users", "user_emails", "messages"].join(", ");
        let q = format!("TRUNCATE TABLE {} RESTART IDENTITY CASCADE;", tables);
        let _ = diesel::sql_query(q)
            .execute(conn)
            .expect("Failed to truncate");
    }
}
