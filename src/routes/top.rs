use rocket::http::Status;
use response::Response;

#[get("/")]
pub fn index() -> Response {
    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({"message": "Eloquentlog ;)"}))
}
