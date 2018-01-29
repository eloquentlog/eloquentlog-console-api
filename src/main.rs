#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rocket::Request;
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

#[error(404)]
fn not_found(req: &Request) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("path", req.uri().as_str());
    Template::render("errors/404", &ctx)
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index, assets])
        .attach(Template::fairing())
        .catch(errors![not_found])
}

fn main() {
    rocket().launch();
}
