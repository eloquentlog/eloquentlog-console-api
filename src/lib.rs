//! Eloquentlog Backend API
//!
//! This is an API for a web frontend of Eloquentlog.
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate accord;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use(error, info, warn)]
extern crate slog;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
extern crate parking_lot;
#[cfg(test)]
#[macro_use]
extern crate rusty_fork;

use std::collections::HashMap;

mod response;
mod validation;
mod service;
mod schema;
mod util;

pub mod db;
pub mod mq;
pub mod ss;

pub mod config;
pub mod job;
pub mod logger;
pub mod mailer;
pub mod model;
pub mod request;
pub mod route;

// macros

#[macro_export]
macro_rules! hashmap(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = HashMap::new();
            $(m.insert($key, $value);)+
            m
        }
    };
);

#[macro_export]
macro_rules! fnvhashmap(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::fnv::FnvHashMap::default();
            $(m.insert($key, $value);)+
            m
        }
    };
);

// returns a sorted vec by namespace.
pub fn routes() -> Vec<(&'static str, Vec<rocket::Route>)> {
    let mut r = vec![
        (
            "/_", // only for web-console
            routes![
                route::activation::preflight::activate,
                route::activation::activate,
                route::authentication::preflight::login,
                route::authentication::preflight::logout,
                route::authentication::preignition::login,
                route::authentication::login,
                route::authentication::logout,
                route::password_reset::preflight::request,
                route::password_reset::preflight::verify_update,
                route::password_reset::preignition::request,
                route::password_reset::preignition::update,
                route::password_reset::request,
                route::password_reset::verify,
                route::password_reset::update,
                route::registration::preflight::deregister,
                route::registration::preflight::register,
                route::registration::preignition::register,
                route::registration::preignition::deregister,
                route::registration::deregister,
                route::registration::register,
                route::health::check,
            ],
        ),
        (
            "/v1", // public console api
            routes![
                route::access_token::preflight::del,
                route::access_token::preflight::dump,
                route::access_token::preflight::hset_state,
                route::access_token::preflight::append,
                route::access_token::preflight::lrange,
                route::access_token::del,
                route::access_token::dump,
                route::access_token::hset_state,
                route::access_token::append,
                route::access_token::lrange,
                route::message::preflight::append,
                route::message::preflight::lrange,
                route::message::append,
                route::message::lrange,
                route::namespace::preflight::hgetall,
                route::namespace::hgetall,
                route::health::check,
            ],
        ),
    ];
    r.sort_by(|a, b| a.0.cmp(b.0));
    r
}

pub fn server() -> rocket::Rocket {
    let r: HashMap<&str, Vec<_>> = routes().iter().cloned().collect();
    rocket::ignite()
        .mount("/_", r["/_"].clone())
        .mount("/v1", r["/v1"].clone())
        .register(catchers![
            route::error::bad_request,
            route::error::internal_server_error,
            route::error::not_found,
            route::error::unauthorized,
            route::error::unprocessable_entity,
        ])
}
