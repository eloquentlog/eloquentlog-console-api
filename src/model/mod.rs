//! Model entities and SQL types.
//!
//! SQL types are imported publicly in each model entities.

// sql types
mod log_level;
mod log_format;
mod user_email_activation_state;
mod user_email_role;
mod user_state;
mod user_reset_password_state;

// non-persistent (deciduous) entities
pub mod ticket;

// models
pub mod message;
pub mod user;
pub mod user_email;

use diesel::prelude::*;
use diesel::PgConnection;

use config::Config;

pub fn establish_connection(config: &Config) -> PgConnection {
    PgConnection::establish(&config.database_url).unwrap_or_else(|_| {
        panic!("Error connecting to : {}", &config.database_url)
    })
}

#[cfg(test)]
pub mod test {
    use super::*;

    use std::panic::{self, AssertUnwindSafe};

    use dotenv::dotenv;

    use logger::{Logger, get_logger};

    /// A test runner
    pub fn run<T>(test: T)
    where T: FnOnce(&PgConnection, &Config, &Logger) -> () + panic::UnwindSafe
    {
        // TODO: remove dotenv from here
        dotenv().ok();
        let config = Config::from("testing").unwrap();
        let conn = establish_connection(&config);
        let logger = get_logger(&config);

        let _: std::result::Result<(), diesel::result::Error> =
            conn.build_transaction().read_write().run(|| {
                setup(&conn);

                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    test(&conn, &config, &logger)
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
    }
}
