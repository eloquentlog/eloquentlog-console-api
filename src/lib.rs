#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

//! Eloquentlog Backend API
//!
//! This is an API for a web frontend of Eloquentlog.

#[macro_use]
extern crate accord;

extern crate bcrypt;
extern crate chrono;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate lazy_static;

extern crate oppgave;
extern crate redis;
extern crate regex;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

extern crate rocket_slog;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use(error, info)]
extern crate slog;

extern crate sloggers;

mod logger;
mod response;
mod request;
mod validation;
mod route;
mod schema;

pub mod config;
pub mod db;
pub mod job;
pub mod model;

use rocket::config::{Config as RocketConfig, Environment, LoggingLevel};

pub fn server(c: &config::Config) -> rocket::Rocket {
    let logger = logger::get_logger(c);

    // disable default logger
    let rocket_config = RocketConfig::build(Environment::Development)
        .log_level(LoggingLevel::Off)
        .finalize()
        .unwrap();

    rocket::custom(rocket_config)
        .mount("/", routes![route::top::index])
        .mount(
            "/_api",
            routes![
                route::authentication::login,
                route::authentication::logout,
                route::registration::register,
                route::registration::deregister,
                route::message::get,
                route::message::post,
                route::message::put,
            ],
        )
        .attach(logger)
        .register(catchers![route::error::not_found])
}
