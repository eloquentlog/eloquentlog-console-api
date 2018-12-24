use rocket::Request;
use rocket::http::Status;

use response::Response;

#[catch(404)]
pub fn not_found(req: &Request) -> Response {
    Response {
        status: Status::NotFound,
        data: json!({
            "data": {
                "message": format!("{path} Not Found", path=req.uri().path()),
            }
        }),
    }
}
