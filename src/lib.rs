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

extern crate fourche;

extern crate jsonwebtoken;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate parking_lot;

extern crate r2d2_redis;
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
mod schema;
mod util;

pub mod db;
pub mod mq;

pub mod config;
pub mod job;
pub mod logger;
pub mod model;
pub mod request;
pub mod route;

use rocket_slog::SlogFairing;

pub fn server(c: &config::Config) -> rocket::Rocket {
    let logger = logger::get_logger(c);

    rocket::ignite()
        .mount("/", routes![route::top::index])
        .mount(
            "/_api",
            routes![
                route::authentication::login,
                route::authentication::login_options,
                route::authentication::logout,
                route::message::get,
                route::message::post,
                route::message::put,
                route::password_reset::options,
                route::password_reset::request,
                route::password_reset::update,
                route::password_reset::verify,
                route::registration::register,
                route::registration::register_options,
                route::registration::deregister,
                route::user::activate,
                route::user::activate_options,
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
