use std::collections::HashMap;

use rocket_contrib::Template;

#[get("/")]
fn index() -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("title", "Eloquentlog;)");
    Template::render("index", &ctx)
}
