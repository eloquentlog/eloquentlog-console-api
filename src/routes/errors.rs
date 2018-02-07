use std::collections::HashMap;

use rocket::Request;
use rocket_contrib::Template;

#[error(404)]
fn not_found(req: &Request) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("path", req.uri().as_str());
    Template::render("errors/404", &ctx)
}
