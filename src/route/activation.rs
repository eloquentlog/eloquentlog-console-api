use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::user::User;
use crate::model::user_email::UserEmail;
use crate::request::token::verification::VerificationToken;
use crate::response::Response;
use crate::service::account_activator::AccountActivator;

pub mod preflight {
    use rocket::State;
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;

    use crate::config::Config;
    use crate::response::no_content_for;

    #[options("/activate/<session_id>", rank = 2)]
    pub fn activate<'a>(
        session_id: String,
        config: State<Config>,
        logger: SyncLogger,
    ) -> RawResponse<'a> {
        info!(logger, "session_id: {}", session_id);
        no_content_for("PATCH", &config)
    }
}

#[patch("/activate/<session_id>", rank = 1)]
pub fn activate(
    session_id: String,
    token: VerificationToken,
    db_conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response {
    info!(logger, "session_id: {}", session_id);

    let res: Response = Default::default();

    let activation =
        AccountActivator::<User, UserEmail>::new(&db_conn, &config, &logger)
            .load(&token)
            .map(|a| {
                let _ = a.activate();
                a
            });
    if activation.is_ok() {
        return res.status(Status::Ok);
    }

    res.status(Status::BadRequest).format(json!({
        "message": "The activation link has been expired or is invalid"
    }))
}
