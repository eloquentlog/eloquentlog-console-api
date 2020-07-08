//! Model entities and SQL types.
//!
//! SQL types are imported publicly in each model entities.

// sql types
mod access_token_state;
mod agent_type;
mod log_level;
mod log_format;
mod user_email_identification_state;
mod user_email_role;
mod user_reset_password_state;
mod user_state;

// non-persistent (deciduous) entities
pub mod token;

// models
pub mod access_token;
pub mod message;
pub mod namespace;
pub mod stream;
pub mod user;
pub mod user_email;

use diesel::pg::PgConnection;

use crate::logger::Logger;

// Note
//
// traits:
// - Authenticatable (User)
// - Activatable (User, UserEmail)
// - Verifiable (UserEmail)
//
// claims
// - AuthenticationClaims
// - VerificationClaims
//
// Authentication [authentication_token_{issuer|key_id|secret}]
// * signin ... access_token (volatile)
//
// Registration [verification_token_{issuer|key_id|secret}]
// * signup ... identification_token (initial primary user email)
//
// Verification [verefication_token_{issuer|key_id|secret}]
// * password reset ... reset_password_token (user)
// * identify       ... identification_token (new general user email)
pub trait Activatable {
    fn activate(
        &self,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<(), &'static str>;
}

pub trait Authenticatable {
    fn update_password(
        &mut self,
        new_password: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<(), &'static str>;

    fn verify_password(&self, password: &str) -> bool;
}

pub trait Verifiable<T: Sized> {
    type TokenClaims: token::Claims;

    fn extract_concrete_token(
        token: &str,
        issuer: &str,
        secret: &str,
    ) -> Result<String, &'static str>;

    fn load_by_concrete_token(
        concrete_token: &str,
        conn: &PgConnection,
        logger: &Logger,
    ) -> Result<T, &'static str>;
}

#[cfg(test)]
pub mod test {
    use std::panic::{self, AssertUnwindSafe};

    use diesel::{PgConnection, prelude::*};
    use dotenv::dotenv;

    use crate::config::Config;
    use crate::db::{DbPoolHolder, init_pool_holder};
    use crate::logger::{Logger, get_logger};

    lazy_static! {
        pub static ref CONFIG: Config = {
            dotenv().ok();
            Config::from("testing").unwrap()
        };
    }

    lazy_static! {
        static ref DB_POOL_HOLDER: DbPoolHolder = {
            init_pool_holder(
                &CONFIG.database_url,
                CONFIG.database_max_pool_size,
            )
        };
    }

    /// A test runner
    pub fn run<T>(test: T)
    where T: FnOnce(&PgConnection, &Config, &Logger) + panic::UnwindSafe {
        let conn = DB_POOL_HOLDER.get().expect("database connection");
        let logger = get_logger(&CONFIG);

        let done = conn
            .build_transaction()
            .read_write()
            .run::<(), diesel::result::Error, _>(|| {
                setup(&conn);

                let result = panic::catch_unwind(AssertUnwindSafe(|| {
                    test(&conn, &CONFIG, &logger);
                }));

                if result.is_err() {
                    return Err(diesel::result::Error::RollbackTransaction);
                }

                teardown(&conn);
                Ok(())
            });
        assert!(done.is_ok());
    }

    fn setup(conn: &PgConnection) {
        clean(conn);
    }

    fn teardown(conn: &PgConnection) {
        clean(conn);
    }

    pub fn clean(conn: &PgConnection) {
        let tables = [
            "users",
            "user_emails",
            "access_tokens",
            "messages",
            "namespaces",
            "streams",
        ]
        .join(", ");
        let q = format!("TRUNCATE TABLE {} RESTART IDENTITY CASCADE;", tables);
        let _ = diesel::sql_query(q)
            .execute(conn)
            .expect("Failed to truncate");
    }
}
