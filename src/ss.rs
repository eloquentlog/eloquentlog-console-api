//! The session store and its connection manager.
use std::ops::{Deref, DerefMut};

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use r2d2_redis::{
    r2d2::Pool, r2d2::PooledConnection, redis, RedisConnectionManager,
};

pub type SsPool = Pool<RedisConnectionManager>;
pub type SsPooledConn = PooledConnection<RedisConnectionManager>;

pub struct SsConn(pub SsPooledConn);

// See DbPoolHolder
#[derive(Clone)]
pub struct SsPoolHolder {
    pool: SsPool,
}

impl SsPoolHolder {
    pub fn get(&self) -> Option<SsPooledConn> {
        self.pool.get().ok()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for SsConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<SsConn, ()> {
        let holder = request.guard::<State<SsPoolHolder>>()?;
        match holder.get() {
            Some(conn) => Outcome::Success(SsConn(conn)),
            None => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for SsConn {
    type Target = redis::Connection;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for SsConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

// Initializes session store connection pool holder
pub fn init_pool_holder(
    session_store_url: &str,
    max_size: u32,
) -> SsPoolHolder {
    let connection_manager =
        RedisConnectionManager::new(session_store_url).unwrap();
    let pool = Pool::builder()
        .max_size(max_size)
        .build(connection_manager)
        .expect("session store pool");
    SsPoolHolder { pool }
}
