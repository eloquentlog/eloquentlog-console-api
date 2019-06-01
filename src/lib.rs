#![feature(
    proc_macro_hygiene,
    decl_macro,
    custom_attribute,
    type_alias_enum_variants
)]

//! Eloquentlog Backend API
//!
//! This is an API for a web frontend of Eloquentlog.

#[macro_use]
extern crate accord;

extern crate bcrypt;
extern crate chrono;
extern crate dotenv;

#[macro_use]
extern crate diesel;

extern crate jsonwebtoken;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

extern crate oppgave;

#[cfg(test)]
extern crate parking_lot;

extern crate rand;
extern crate redis;
extern crate regex;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

extern crate rocket_slog;

#[cfg(test)]
#[macro_use]
extern crate rusty_fork;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use(error, info, warn)]
extern crate slog;

extern crate sloggers;
extern crate uuid;

mod response;
mod validation;
mod route;
mod schema;
mod util;

pub mod config;
pub mod db;
pub mod job;
pub mod logger;
pub mod model;
pub mod request;

use rocket::config::{Config as RocketConfig, Environment, LoggingLevel};
use rocket_slog::SlogFairing;

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
                route::registration::activate,
                route::registration::register,
                route::registration::deregister,
                route::message::get,
                route::message::post,
                route::message::put,
            ],
        )
        .manage(c.clone()) // TODO: not good?
        .attach(SlogFairing::new(logger))
        .register(catchers![
            route::error::bad_request,
            route::error::not_found,
            route::error::unprocessable_entity,
        ])
}
