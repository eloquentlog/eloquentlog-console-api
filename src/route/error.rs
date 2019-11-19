use rocket::Request;
use rocket::http::Status;

use crate::response::Response;

#[catch(400)]
pub fn bad_request<'a>(_req: &Request) -> Response<'a> {
    Response {
        cookies: vec![],
        status: Status::BadRequest,
        data: json!({
            "data": {
                "message": "The request header/body is invalid".to_string(),
            }
        }),
    }
}

#[catch(404)]
pub fn not_found<'a>(req: &Request) -> Response<'a> {
    Response {
        cookies: vec![],
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
        cookies: vec![],
        status: Status::UnprocessableEntity,
        data: json!({
            "data": {
                "message": "The input is invalid".to_string(),
            }
        }),
    }
}
