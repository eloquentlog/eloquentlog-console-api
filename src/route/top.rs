use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use config::Config;
use response::Response;

#[get("/")]
pub fn index(_logger: SyncLogger, _state: State<Config>) -> Response {
    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({"message": "Eloquentlog ;)"}))
}
