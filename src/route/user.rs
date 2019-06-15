use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use response::{Response, no_content_for};

#[options("/user/activate?<token>")]
pub fn activate_options<'a>(
    token: String,
    logger: SyncLogger,
) -> RawResponse<'a>
{
    info!(logger, "token: {}", token);
    no_content_for("GET")
}

#[get("/user/activate?<token>", format = "json")]
pub fn activate(token: String, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    // TODO
    info!(logger, "token: {}", token);

    res.status(Status::Ok)
}
