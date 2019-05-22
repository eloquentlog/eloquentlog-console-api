extern crate dotenv;
extern crate rocket;

extern crate eloquentlog_backend_api;

use std::env;

use dotenv::dotenv;

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
    dotenv().ok();

    let name = get_env();
    let config = Config::from(name.as_str()).expect("Failed to get config");

    // database
    let connection_pool = init_pool(&config.database_url);

    server(config).manage(connection_pool).launch();
}
