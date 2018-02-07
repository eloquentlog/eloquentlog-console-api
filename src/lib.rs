#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::Template;

mod routes {
    pub mod index;
    pub mod assets;
    pub mod errors;
}

pub fn app() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![
            routes::index::index,
            routes::assets::assets
        ])
        .attach(Template::fairing())
        .catch(errors![
            routes::errors::not_found
        ])
}
