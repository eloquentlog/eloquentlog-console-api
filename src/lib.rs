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
            let mut m = ::std::collections::HashMap::new();
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

pub fn server() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![route::top::index])
        .mount(
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
                route::access_token::preflight::generate,
                route::access_token::preflight::list,
                route::access_token::generate,
                route::access_token::list,
                route::message::preflight::add,
                route::message::preflight::lrange,
                route::message::preflight::put,
                route::message::add,
                route::message::lrange,
                route::message::put,
                route::user::preflight::activate,
                route::user::activate,
            ],
        )
        .register(catchers![
            route::error::bad_request,
            route::error::not_found,
            route::error::unprocessable_entity,
        ])
}
