extern crate dotenv;
extern crate oppgave;
extern crate redis;

extern crate eloquentlog_backend_api;

use std::env;
use dotenv::dotenv;
use oppgave::Queue;
use redis::Client;

use eloquentlog_backend_api::job;
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

    // redis
    let client = Client::open(config.queue_url.as_str()).unwrap();
    let conn = client.get_connection().unwrap();

    let worker = Queue::new("default".into(), conn);

    while let Some(job) = worker.next::<job::Job>() {
        if job.is_err() {
            continue;
        }
        let actual_job = job.unwrap();

        println!("job: {}", actual_job.id);
    }
}
