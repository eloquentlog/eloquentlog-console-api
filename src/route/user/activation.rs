use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use request::token::verification::VerificationToken;
use response::{Response, no_content_for};
use service::activator::UserActivator;

#[options("/user/activate")]
pub fn activate_options<'a>() -> RawResponse<'a> {
    no_content_for("PATCH")
}

#[patch("/user/activate", format = "json")]
pub fn activate(
    verification_token: VerificationToken,
    db_conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    let activator = UserActivator::new(&db_conn, &config, &logger);
    if activator.activate(&verification_token).is_ok() {
        return res.status(Status::Ok);
    }

    res.status(Status::BadRequest).format(json!({
        "message": "The activation link has been expired or is invalid"
    }))
}
