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
use eloquentlog_backend_api::db::establish_connection;
use eloquentlog_backend_api::job::Job;
use eloquentlog_backend_api::logger;

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
    let config = Config::from(name.as_str()).expect("failed to get config");

    // redis
    let client = Client::open(config.queue_url.as_str()).unwrap();
    let mq_conn = client.get_connection().unwrap();

    // postgresql
    let db_conn = establish_connection(&config);

    let logger = logger::get_logger(&config);
    let queue = Queue::new("default", &mq_conn);

    loop {
        match queue.dequeue::<Job<i64>>() {
            Ok(job) => {
                info!(
                    logger,
                    "kind: {}, args: {:?}",
                    job.kind,
                    job.args.as_slice()
                );
                job.invoke(&db_conn, &logger, &config);
            },
            Err(e) => {
                error!(logger, "err: {}", e);
                break;
            },
        }
    }
}
