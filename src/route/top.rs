use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::model::user::User;
use crate::response::Response;

#[get("/")]
pub fn index<'a>(
    user: &User,
    _logger: SyncLogger,
    _state: State<Config>,
) -> Response<'a>
{
    let res = Response {
        cookies: vec![],
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({
        "message": format!("Welcome to Eloquentlog, {} ;)", user.username)
    }))
}
