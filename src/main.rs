extern crate eloquentlog_backend_api;
extern crate dotenv;

use std::env;
use dotenv::dotenv;

use eloquentlog_backend_api::app;

fn main() {
    dotenv().ok();

    let env_ = match env::var("ENV") {
        Ok(v) => v.to_lowercase(),
        Err(_) => String::from("development"),
    };

    app(env_.as_str()).launch();
}
