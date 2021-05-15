//! The message queue and its connection manager.
use std::ops::{Deref, DerefMut};

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use r2d2_redis::{
    r2d2::Pool, r2d2::PooledConnection, redis, RedisConnectionManager,
};

pub type MqPool = Pool<RedisConnectionManager>;
pub type MqPooledConn = PooledConnection<RedisConnectionManager>;

pub struct MqConn(pub MqPooledConn);

// See DbPoolHolder
#[derive(Clone)]
pub struct MqPoolHolder {
    pool: MqPool,
}

impl MqPoolHolder {
    pub fn get(&self) -> Option<MqPooledConn> {
        self.pool.get().ok()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for MqConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<MqConn, ()> {
        let holder = request.guard::<State<MqPoolHolder>>()?;
        match holder.get() {
            Some(conn) => Outcome::Success(MqConn(conn)),
            None => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for MqConn {
    type Target = redis::Connection;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for MqConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

// Initializes message queue connection pool holder
pub fn init_pool_holder(
    message_queue_url: &str,
    max_size: u32,
) -> MqPoolHolder {
    let connection_manager =
        RedisConnectionManager::new(message_queue_url).unwrap();
    let pool = Pool::builder()
        .max_size(max_size)
        .build(connection_manager)
        .expect("message queue pool");
    MqPoolHolder { pool }
}
