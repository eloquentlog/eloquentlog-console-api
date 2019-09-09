use std::io::Cursor;

use rocket::http::{Cookie, ContentType, Status};
use rocket_contrib::json::JsonValue;
use rocket::request::Request;
use rocket::response::Responder;
use rocket::response::Response as RawResponse;

use route::{ORIGIN, MAX_AGE, VARY};

#[derive(Debug)]
pub struct Response<'a> {
    pub cookies: Vec<Cookie<'a>>,
    pub status: Status,
    pub data: JsonValue,
}

impl<'a> Default for Response<'a> {
    fn default() -> Self {
        Self {
            cookies: vec![],
            status: Status::Ok,
            data: json!(null),
        }
    }
}

impl<'a> Response<'a> {
    pub fn cookies(mut self, cookies: Vec<Cookie<'a>>) -> Response<'a> {
        self.cookies = cookies;
        self
    }

    pub fn status(mut self, status: Status) -> Response<'a> {
        self.status = status;
        self
    }

    // format its data attribute using json
    pub fn format(mut self, data: JsonValue) -> Response<'a> {
        self.data = data;
        self
    }
}

impl<'r> Responder<'r> for Response<'r> {
    fn respond_to(self, _req: &Request) -> Result<RawResponse<'r>, Status> {
        let mut builder = RawResponse::build();

        builder.status(self.status);
        builder.header(ContentType::JSON);
        if !self.cookies.is_empty() {
            self.cookies.iter().for_each(|c| {
                builder.header(c);
            });
        }
        builder
            .raw_header("Access-Control-Allow-Origin", ORIGIN)
            .raw_header("Access-Control-Allow-Credentials", "true")
            .raw_header("Vary", VARY);

        let body = self.data.to_string();
        builder.sized_body(Cursor::new(body)).ok()
    }
}

/// Returns RawResponse (Rocket's original response) for HTTP 204 No Content to
/// OPTIONS request.
pub fn no_content_for<'a>(methods: &str) -> RawResponse<'a> {
    let mut res = RawResponse::new();
    res.set_header(ContentType::JSON);
    res.set_raw_header("Access-Control-Allow-Credentials", "true");
    res.set_raw_header(
        "Access-Control-Allow-Headers",
        "Authorization,Content-Type,X-Requested-With",
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
