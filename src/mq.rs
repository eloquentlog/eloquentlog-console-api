//! The message queue and its connection manager.
use std::ops::{Deref, DerefMut};

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use r2d2_redis::{r2d2, redis, RedisConnectionManager};

// An alias to connection pool of Redis
pub type MqPool = r2d2::Pool<RedisConnectionManager>;

pub struct MqConn(pub r2d2::PooledConnection<RedisConnectionManager>);

impl<'a, 'r> FromRequest<'a, 'r> for MqConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<MqConn, ()> {
        let pool = request.guard::<State<MqPool>>()?.clone();
        match pool.get() {
            Ok(conn) => Outcome::Success(MqConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
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

// Initializes queue connection pool
pub fn init_pool(queue_url: &str, max_size: u32) -> MqPool {
    let manager = RedisConnectionManager::new(queue_url).unwrap();
    r2d2::Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("queue pool")
}
