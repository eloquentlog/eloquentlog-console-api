use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use response::{Response, no_content_for};

#[options("/password/reset?<token>")]
pub fn options<'a>(
    token: Option<String>,
    logger: SyncLogger,
) -> RawResponse<'a>
{
    if let Some(value) = token {
        info!(logger, "token: {}", value);
        no_content_for("GET,PUT")
    } else {
        no_content_for("POST")
    }
}

#[post("/password/reset", format = "json")]
pub fn request<'a>(logger: SyncLogger) -> Response<'a> {
    let res: Response = Default::default();

    // TODO
    let token = "token";
    info!(logger, "token: {}", token);

    res.format(json!({ "activation_token": token }))
}

#[get("/password/reset?<token>", format = "json")]
pub fn verify<'a>(token: String, logger: SyncLogger) -> Response<'a> {
    let res: Response = Default::default();

    // TODO
    info!(logger, "token: {}", token);

    res.status(Status::Ok)
}

#[put("/password/reset?<token>", format = "json")]
pub fn update<'a>(token: String, logger: SyncLogger) -> Response<'a> {
    let res: Response = Default::default();

    // TODO
    info!(logger, "token: {}", token);

    res.status(Status::Ok)
}
