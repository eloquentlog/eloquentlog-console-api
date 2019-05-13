use rocket::http::Status;
use rocket_slog::SyncLogger;

use response::Response;

#[get("/")]
pub fn index(_logger: SyncLogger) -> Response {
    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({"message": "Eloquentlog ;)"}))
}
