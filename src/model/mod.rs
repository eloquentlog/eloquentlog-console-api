//! Model entities and SQL types.
//!
//! SQL types are imported publicly in each model entities.

// sql types
mod log_level;
mod log_format;
mod user_activation_state;

// models
pub mod message;
pub mod user;

#[cfg(test)]
pub mod test {
    use std::panic::{self, AssertUnwindSafe};

    use diesel::{self, prelude::*};
    use diesel::PgConnection;

    use config::Config;

    /// A test runner
    pub fn run<T>(test: T)
    where T: FnOnce(&PgConnection) -> () + panic::UnwindSafe {
        let conn = establish_connection();

        let _: std::result::Result<(), diesel::result::Error> =
            conn.build_transaction().read_write().run(|| {
                setup(&conn);

                let result =
                    panic::catch_unwind(AssertUnwindSafe(|| test(&conn)));

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

    pub fn establish_connection() -> PgConnection {
        dotenv::dotenv().ok();

        let c = Config::from("testing").unwrap();
        PgConnection::establish(&c.database_url).unwrap_or_else(|_| {
            panic!("Error connecting to : {}", c.database_url)
        })
    }
}
