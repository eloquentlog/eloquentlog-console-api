#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rocket::response::NamedFile;
use rocket_contrib::Template;


#[get("/")]
fn index() -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("title", "Eloquentlog;)");
    Template::render("index", &ctx)
}

#[get("/static/<file..>")]
fn assets(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, assets])
}

fn main() {
    rocket().launch();
}
