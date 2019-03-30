// sql types
mod level;
mod format;
mod activation_state;

// models
pub mod message;
pub mod user;

#[cfg(test)]
mod test {
    use std::panic::{self, AssertUnwindSafe};

    use diesel::{self, prelude::*};
    use diesel::PgConnection;

    use config::Config;

    /// A test runner
    pub fn run<T>(test: T)
    where T: FnOnce(&PgConnection) -> () + panic::UnwindSafe {
        let conn = establish_connection();

        let _: std::result::Result<(), diesel::result::Error> = conn
            .build_transaction()
            .serializable()
            .read_write()
            .run(|| {
                setup(&conn);

                let result =
                    panic::catch_unwind(AssertUnwindSafe(|| test(&conn)));

                teardown(&conn);

                assert!(result.is_ok());
                Ok(())
            });
    }

    fn setup(conn: &PgConnection) {
        truncate_messages(conn);
    }

    fn teardown(conn: &PgConnection) {
        truncate_messages(conn);
    }

    pub fn truncate_messages(conn: &PgConnection) {
        let _ = diesel::sql_query("TRUNCATE TABLE messages;")
            .execute(conn)
            .expect("Failed to truncate");

        let _ =
            diesel::sql_query("ALTER SEQUENCE messages_id_seq RESTART WITH 1;")
                .execute(conn)
                .expect("Failed to reset sequence");
    }

    pub fn establish_connection() -> PgConnection {
        dotenv::dotenv().ok();

        let c = Config::from("testing").unwrap();
        PgConnection::establish(&c.database_url).unwrap_or_else(|_| {
            panic!("Error connecting to : {}", c.database_url)
        })
    }
}
