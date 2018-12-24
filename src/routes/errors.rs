use std::collections::HashMap;

use rocket::Request;
use rocket_contrib::templates::Template;

#[catch(404)]
pub fn not_found(req: &Request) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("path", req.uri().path());
    Template::render("errors/404", &ctx)
}
