use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::user::User;
use crate::model::user_email::UserEmail;
use crate::request::token::verification::VerificationToken;
use crate::response::{Response, no_content_for};
use crate::service::account_activator::AccountActivator;

#[options("/user/activate/<session_id>")]
pub fn activate_preflight<'a>(
    session_id: String,
    logger: SyncLogger,
) -> RawResponse<'a>
{
    info!(logger, "session_id: {}", session_id);
    no_content_for("PATCH")
}

#[patch("/user/activate/<session_id>")]
pub fn activate(
    session_id: String,
    token: VerificationToken,
    db_conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    info!(logger, "session_id: {}", session_id);

    let res: Response = Default::default();

    let activation =
        AccountActivator::<User, UserEmail>::new(&db_conn, &config, &logger)
            .load(&token)
            .and_then(|a| {
                let _ = a.activate();
                Ok(a)
            });
    if activation.is_ok() {
        return res.status(Status::Ok);
    }

    res.status(Status::BadRequest).format(json!({
        "message": "The activation link has been expired or is invalid"
    }))
}
