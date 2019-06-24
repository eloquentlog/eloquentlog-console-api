extern crate dotenv;
extern crate rocket;

extern crate eloquentlog_backend_api;

use std::env;

use dotenv::dotenv;

use eloquentlog_backend_api::server;
use eloquentlog_backend_api::db::init_pool as init_db_pool;
use eloquentlog_backend_api::mq::init_pool as init_mq_pool;
use eloquentlog_backend_api::config::Config;

fn get_env() -> String {
    match env::var("ENV") {
        Ok(ref v) if v == &"test".to_string() => String::from("testing"),
        Ok(v) => v.to_lowercase(),
        Err(_) => String::from("development"),
    }
}

fn main() {
    let name = get_env();

    dotenv().ok();
    let config = Config::from(name.as_str()).expect("Failed to get config");

    // connection pools
    let db_pool = init_db_pool(&config.database_url);
    let mq_pool = init_mq_pool(&config.queue_url);

    server(&config).manage(db_pool).manage(mq_pool).launch();
}
