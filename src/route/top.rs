use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use config::Config;
use model::user::User;
use response::Response;

#[get("/")]
pub fn index(
    user: &User,
    _logger: SyncLogger,
    _state: State<Config>,
) -> Response
{
    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({
        "message": format!("Welcome to Eloquentlog, {} ;)", user.username)
    }))
}
