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

mod db;
mod response;
mod request;
mod validation;

mod route {
    pub mod auth;
    pub mod error;
    pub mod message;
    pub mod top;
}

pub mod config;
pub mod model;

pub fn app(env_name: &str) -> rocket::Rocket {
    let config_name = match env_name {
        "test" => "testing",
        _ => env_name,
    };

    let config = config::Config::from(config_name).unwrap();
    let pool = db::init_pool(&config.database_url);

    rocket::ignite()
        .manage(pool)
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
