#![feature(proc_macro_hygiene, decl_macro)]

//! Eloquentlog Backend API
//!
//! This is an API for a web frontend of Eloquentlog.

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
        ("/", routes![route::top::index]),
        (
            "/_api",
            routes![
                // foundation
                route::authentication::preflight::login,
                route::authentication::preflight::logout,
                route::authentication::login,
                route::authentication::logout,
                route::password_reset::preflight::request,
                route::password_reset::preflight::verify_update,
                route::password_reset::request,
                route::password_reset::verify,
                route::password_reset::update,
                route::registration::preflight::deregister,
                route::registration::preflight::register,
                route::registration::deregister,
                route::registration::register,
                // resource
                route::access_token::preflight::lpop,
                route::access_token::preflight::lrange,
                route::access_token::lpop,
                route::access_token::lrange,
                route::message::preflight::add,
                route::message::preflight::lrange,
                route::message::preflight::put,
                route::message::add,
                route::message::lrange,
                route::message::put,
                route::user::preflight::activate,
                route::user::activate,
            ],
        ),
    ];
    r.sort_by(|a, b| a.0.cmp(b.0));
    r
}

pub fn server() -> rocket::Rocket {
    let r: HashMap<&str, Vec<_>> = routes().iter().cloned().collect();
    rocket::ignite()
        .mount("/", r["/"].clone())
        .mount("/_api", r["/_api"].clone())
        .register(catchers![
            route::error::bad_request,
            route::error::not_found,
            route::error::unprocessable_entity,
        ])
}
