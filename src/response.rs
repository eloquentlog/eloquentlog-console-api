use std::io::Cursor;

use rocket::http::{ContentType, Status};
use rocket_contrib::json::JsonValue;
use rocket::request::Request;
use rocket::response::Responder;
use rocket::response::Response as RawResponse;

use route::{ORIGIN, MAX_AGE, VARY};

#[derive(Debug)]
pub struct Response {
    pub status: Status,
    pub data: JsonValue,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: Status::Ok,
            data: json!(null),
        }
    }
}

impl Response {
    pub fn status(mut self, status: Status) -> Response {
        self.status = status;
        self
    }

    // format its data attribute using json
    pub fn format(mut self, data: JsonValue) -> Response {
        self.data = data;
        self
    }
}

impl<'r> Responder<'r> for Response {
    fn respond_to(self, _req: &Request) -> Result<RawResponse<'r>, Status> {
        let body = self.data;

        RawResponse::build()
            .status(self.status)
            .sized_body(Cursor::new(body.to_string()))
            .header(ContentType::JSON)
            .raw_header("Access-Control-Allow-Origin", ORIGIN)
            .raw_header("Vary", VARY)
            .ok()
    }
}

/// Returns RawResponse (Rocket's original response) for HTTP 204 No Content to
/// OPTIONS request.
pub fn no_content_for<'a>(methods: &str) -> RawResponse<'a> {
    let mut res = RawResponse::new();
    res.set_header(ContentType::JSON);
    res.set_raw_header(
        "Access-Control-Allow-Headers",
        "Content-Type,Authorization",
    );
    res.set_raw_header(
        "Access-Control-Allow-Methods",
        ["OPTIONS", methods].join(","),
    );
    res.set_raw_header("Access-Control-Allow-Origin", ORIGIN);
    res.set_raw_header("Access-Control-Max-Age", MAX_AGE);
    res.set_raw_header("Vary", VARY);
    res.set_status(Status::NoContent);
    res
}
