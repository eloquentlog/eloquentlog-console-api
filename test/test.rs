extern crate rocket;
extern crate dotenv;
extern crate diesel;

extern crate eloquentlog_backend_api;

mod login;
mod message;
mod error;
mod top;

use std::panic;

use dotenv::dotenv;

fn run_test<T>(test: T)
where T: FnOnce() -> () + panic::UnwindSafe {
    setup();
    let result = panic::catch_unwind(test);
    teardown();
    assert!(result.is_ok())
}

fn setup() {
    dotenv().ok();
}

fn teardown() {}
