extern crate dotenv;
extern crate fourche;
extern crate native_tls;
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
use eloquentlog_backend_api::logger::get_logger;

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
    let client = Client::open(config.message_queue_url.as_str()).unwrap();
    let mut mq_conn = client.get_connection().unwrap();

    // postgresql
    let db_conn = establish_connection(&config);

    let logger = get_logger(&config);
    let mut queue = Queue::new("default", &mut mq_conn);

    loop {
        match queue.dequeue::<Job<String>>() {
            Ok(job) => {
                info!(
                    logger,
                    "kind: {}, args: {:?}",
                    job.kind,
                    job.args.as_slice()
                );
                job.invoke(&db_conn, &config, &logger);
            },
            Err(e) => {
                error!(logger, "err: {}", e);
                break;
            },
        }
    }
}
