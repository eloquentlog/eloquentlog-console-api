use rocket::http::Status;
use response::Response;

#[get("/")]
pub fn index() -> Response {
    Response {
        status: Status::Ok,
        data: json!({
            "message": "Eloquentlog ;)",
        })
    }
}
