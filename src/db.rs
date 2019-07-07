//! The database connection and its manager.
use std::ops::Deref;

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use diesel::{PgConnection, prelude::*};
use diesel::r2d2::{self, ConnectionManager};

use config::Config;

// An alias to connection pool of PostgreSQL
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<DbPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Returns a single connection.
pub fn establish_connection(config: &Config) -> PgConnection {
    PgConnection::establish(&config.database_url).unwrap_or_else(|_| {
        panic!("Error connecting to : {}", &config.database_url)
    })
}

// Initializes db connection pool.
pub fn init_pool(database_url: &str, max_size: u32) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(max_size)
        .build(manager)
        .unwrap()
}
