//! An utility prints routes.
#![feature(rustc_private)]

use std::env;

use dotenv::dotenv;
use proctitle::set_title;

use eloquentlog_console_api::routes;
use eloquentlog_console_api::config::Config;
use eloquentlog_console_api::logger::get_logger;

fn get_env() -> String {
    match env::var("ENV") {
        Ok(ref v) if v == &"test".to_string() => String::from("testing"),
        Ok(v) => v.to_lowercase(),
        Err(_) => String::from("development"),
    }
}

// FIXME: consider about the type of uri
fn format_route(
    r: rocket::Route,
    ns: &'static str,
) -> (&str, &str, String, String) {
    let uri = if ns != "/" {
        format!("{}{}", ns, r.uri)
    } else {
        r.uri.to_string()
    };
    (
        r.method.as_str(),
        r.name.unwrap_or(""),
        r.rank.abs().to_string(),
        uri,
    )
}

fn main() {
    set_title("eloquentlog: router");
    let name = get_env();

    dotenv().ok();
    let config = Config::from(name.as_str()).expect("failed to get config");

    let _logger = get_logger(&config);

    let mut buf: Vec<(&str, &str, String, String)> = vec![];

    for route in routes() {
        buf.push(("Method", "Name", "Rank".to_string(), "URI".to_string()));

        for r in route.1 {
            buf.push(format_route(r, route.0));
        }
        buf.push(("", "", "".to_string(), "".to_string()));
    }

    for b in buf {
        println!("{:<7} {:<13} {:<4} {}", b.0, b.1, b.2, b.3);
    }
}

#[cfg(test)]
mod test {
    use rocket::handler::Outcome;
    use rocket::{Request, Data};
    use rocket::http::Method;

    use super::*;

    fn handler<'r>(request: &'r Request, _data: Data) -> Outcome<'r> {
        Outcome::from(request, "")
    }

    #[test]
    fn test_format_route() {
        let result = format_route(
            rocket::Route::new(Method::Get, "/foo?bar=baz&<zoo>", handler),
            "/",
        );
        assert_eq!(
            result,
            ("GET", "", "6".to_string(), "/foo?bar=baz&<zoo>".to_string())
        );

        let result = format_route(
            rocket::Route::new(Method::Post, "/foo?<zoo>", handler),
            "/",
        );
        assert_eq!(
            result,
            ("POST", "", "5".to_string(), "/foo?<zoo>".to_string())
        );

        let result =
            format_route(rocket::Route::new(Method::Put, "/", handler), "/");
        assert_eq!(result, ("PUT", "", "4".to_string(), "/".to_string()));

        let result = format_route(
            rocket::Route::new(Method::Delete, "/foo/<bar>?blue", handler),
            "/api",
        );
        assert_eq!(
            result,
            (
                "DELETE",
                "",
                "3".to_string(),
                "/api/foo/<bar>?blue".to_string()
            )
        );

        let result = format_route(
            rocket::Route::new(Method::Options, "/<bar>?<blue>", handler),
            "/api",
        );
        assert_eq!(
            result,
            (
                "OPTIONS",
                "",
                "2".to_string(),
                "/api/<bar>?<blue>".to_string()
            )
        );

        let result = format_route(
            rocket::Route::new(Method::Patch, "/<bar>/foo/<baz..>", handler),
            "/api",
        );
        assert_eq!(
            result,
            (
                "PATCH",
                "",
                "1".to_string(),
                "/api/<bar>/foo/<baz..>".to_string()
            )
        );
    }
}
