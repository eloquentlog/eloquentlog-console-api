#![feature(proc_macro_hygiene, decl_macro, custom_attribute)]

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

extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;

pub mod config;
pub mod db;
pub mod response;
pub mod model;

mod routes {
    pub mod auth;
    pub mod index;
    pub mod errors;
}

pub fn app(env_name: &str) -> rocket::Rocket {
    let config_name = match env_name {
        "test" => "testing",
        _ => env_name,
    };

    let config = config::Config::from(config_name).unwrap();
    let pool = db::init_pool(&config.database_url);

    rocket::ignite()
        .manage(pool)
        .mount("/", routes![routes::auth::login, routes::index::index,])
        .register(catchers![routes::errors::not_found])
}
