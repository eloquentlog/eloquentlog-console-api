#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

#[macro_use]
extern crate accord;
extern crate chrono;

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

#[macro_use]
extern crate diesel;

mod response;
mod request;
mod validation;
mod schema;

mod route {
    pub mod auth;
    pub mod error;
    pub mod message;
    pub mod top;
}

pub mod db;
pub mod config;
pub mod model;

pub fn app() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![route::top::index, route::auth::login,])
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
