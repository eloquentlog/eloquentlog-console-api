extern crate dotenv;
extern crate fourche;
extern crate redis;

#[macro_use(error, info)]
extern crate slog;

extern crate eloquentlog_backend_api;

use std::env;

use dotenv::dotenv;
use fourche::queue::Queue;
use redis::Client;

use eloquentlog_backend_api::config::Config;
use eloquentlog_backend_api::logger;
use eloquentlog_backend_api::job;

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

    // redis
    let client = Client::open(config.queue_url.as_str()).unwrap();
    let conn = client.get_connection().unwrap();

    let logger = logger::get_logger(&config);
    let queue = Queue::new("default", &conn);

    loop {
        // TODO: Result
        match queue.dequeue::<job::Job>() {
            Ok(job) => info!(logger, "job: {}", job.id),
            _ => {
                error!(logger, "err");
                break;
            },
        }
    }
}
