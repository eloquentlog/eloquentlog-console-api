use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::user_email::UserEmail;
use model::token::ActivationClaims;
use request::user::UserActivation as RequestData;
use response::{Response, no_content_for};

#[options("/user/activate")]
pub fn activate_options<'a>() -> RawResponse<'a> {
    no_content_for("PUT")
}

#[put("/user/activate", data = "<data>", format = "json")]
pub fn activate(
    data: RequestData,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    match UserEmail::find_by_token::<ActivationClaims>(
        &data.token,
        &config.activation_token_issuer,
        &config.activation_token_secret,
        &conn,
        &logger,
    ) {
        Some(ref user_email) if user_email.is_primary() => {
            let _ = user_email.activate(&conn, &logger);
            info!(
                logger,
                "an user email({}) has been activate (granted: {})",
                user_email.role,
                user_email
                    .activation_token_granted_at
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S"),
            );
            res.status(Status::Ok)
        },
        _ => {
            warn!(logger, "activation failed: token {}", data.token);

            res.status(Status::BadRequest).format(json!({
                "message": "The credentials you've entered is expired/incorrect"
            }))
        },
    }
}
