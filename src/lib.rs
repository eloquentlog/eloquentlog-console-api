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
extern crate regex;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate uuid;

mod response;
mod request;
mod validation;
mod route;
mod schema;

pub mod db;
pub mod config;
pub mod model;

pub fn app() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            routes![
                route::top::index,
                route::auth::login,
                route::user::register,
            ],
        )
        .mount(
            "/api",
            routes![
                route::message::get,
                route::message::post,
                route::message::put,
            ],
        )
        .register(catchers![route::error::not_found])
}
