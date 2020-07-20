use rocket::Request;
use rocket::http::{Cookies, Status};

use crate::response::Response;

#[catch(400)]
pub fn bad_request<'a>(_req: &Request) -> Response<'a> {
    Response {
        cookies: Cookies::empty(),
        status: Status::BadRequest,
        data: json!({
            "data": {
                "message": "The request header/body is invalid".to_string(),
            }
        }),
    }
}

#[catch(401)]
pub fn unauthorized<'a>(_req: &Request) -> Response<'a> {
    Response {
        cookies: Cookies::empty(),
        status: Status::Unauthorized,
        data: json!({
            "data": {
                "message": "The request is not allowed".to_string(),
            }
        }),
    }
}

#[catch(403)]
pub fn forbidden<'a>(_req: &Request) -> Response<'a> {
    Response {
        cookies: Cookies::empty(),
        status: Status::Unauthorized,
        data: json!({
            "data": {
                "message": "The request is not prohibited".to_string(),
            }
        }),
    }
}

#[catch(404)]
pub fn not_found<'a>(req: &Request) -> Response<'a> {
    Response {
        cookies: Cookies::empty(),
        status: Status::NotFound,
        data: json!({
            "data": {
                "message": format!("'{path}' is not found", path=req.uri().path()),
            }
        }),
    }
}

#[catch(422)]
pub fn unprocessable_entity<'a>(_req: &Request) -> Response<'a> {
    Response {
        cookies: Cookies::empty(),
        status: Status::UnprocessableEntity,
        data: json!({
            "data": {
                "message": "The input is invalid".to_string(),
            }
        }),
    }
}

#[catch(500)]
pub fn internal_server_error<'a>(_req: &Request) -> Response<'a> {
    Response {
        cookies: Cookies::empty(),
        status: Status::InternalServerError,
        data: json!({
            "data": {
                "message": "Internal server error occured".to_string(),
            }
        }),
    }
}
