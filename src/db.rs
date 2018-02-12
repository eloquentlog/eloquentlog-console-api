use r2d2;
use diesel::PgConnection;
use r2d2_diesel::ConnectionManager;


pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// initializes db connection pool
pub fn init_pool(database_url: &str) -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::new(manager).expect("db pool")
}
