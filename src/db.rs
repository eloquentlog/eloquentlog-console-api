//! The database connection and its manager.
use std::ops::Deref;

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use diesel::{PgConnection, prelude::*};
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use config::Config;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbPooledConn = PooledConnection<ConnectionManager<PgConnection>>;

pub struct DbConn(pub DbPooledConn);

// NOTE:
// The type alias is not acceptable in this case because we have mulitple
// connection pools in same type and `rocket::Rocket::manage` can handle state
// per type basis. so, if we use type alias for this, it would say
// "Error: State for this type is already being managed!".
//
// See also:
// https://github.com/SergioBenitez/Rocket/issues/1053
#[derive(Clone)]
pub struct DbPoolHolder {
    pool: DbPool,
}

impl DbPoolHolder {
    pub fn get(&self) -> Option<DbPooledConn> {
        self.pool.get().ok()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let holder = req.guard::<State<DbPoolHolder>>()?;
        match holder.get() {
            Some(conn) => Outcome::Success(DbConn(conn)),
            None => Outcome::Failure((Status::ServiceUnavailable, ())),
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

// Initializes db connection pool holder.
pub fn init_pool_holder(database_url: &str, max_size: u32) -> DbPoolHolder {
    let connection_manager =
        ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(max_size)
        .build(connection_manager)
        .expect("database pool");
    DbPoolHolder { pool }
}
