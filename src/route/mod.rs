pub mod authentication;
pub mod error;
pub mod message;
pub mod registration;
pub mod top;

use rocket::response::Response as RawResponse;
use rocket::http::{ContentType, Status};

// TODO
pub const ORIGIN: &str = "http://127.0.0.1:3000";
pub const MAX_AGE: &str = "10800"; // 3 hours
pub const VARY: &str = "Accept-Encoding,Origin";

#[route(OPTIONS, path = "/login")]
pub fn options_login<'a>() -> RawResponse<'a> {
    let mut res = RawResponse::new();
    res.set_header(ContentType::JSON);
    res.set_raw_header("Access-Control-Allow-Headers", "Content-Type");
    res.set_raw_header("Access-Control-Allow-Methods", "GET,POST,PUT,DELETE");
    res.set_raw_header("Access-Control-Allow-Origin", ORIGIN);
    res.set_raw_header("Access-Control-Max-Age", MAX_AGE);
    res.set_raw_header("Vary", VARY);
    res.set_status(Status::NoContent);
    res
}
