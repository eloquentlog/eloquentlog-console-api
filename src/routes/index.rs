use std::collections::HashMap;

use rocket_contrib::templates::Template;

#[get("/")]
pub fn index() -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("title", "Eloquentlog;)");
    Template::render("index", &ctx)
}
