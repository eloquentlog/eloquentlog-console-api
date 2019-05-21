extern crate dotenv;
extern crate rocket;

extern crate eloquentlog_backend_api;

use std::env;

use eloquentlog_backend_api::server;
use eloquentlog_backend_api::db::init_pool;
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
    let c = Config::from(name.as_str()).expect("Failed to get config");

    // database
    let connection_pool = init_pool(&c.database_url);

    server(&c).manage(connection_pool).launch();
}
