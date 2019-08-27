use chrono::Utc;
use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::user::User;
use model::token::{AuthorizationClaims, Claims, TokenData};
use request::user::UserSignin as RequestData;
use response::{Response, no_content_for};

#[options("/signin")]
pub fn signin_options<'a>() -> RawResponse<'a> {
    no_content_for("POST")
}

#[post("/signin", data = "<data>", format = "json")]
pub fn signin(
    data: RequestData,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    match User::find_by_email_or_uuid(&data.username, &conn, &logger) {
        Some(ref user) if user.verify_password(&data.password) => {
            // TODO:
            // set valid expires_at and impl review mechanism (check also
            // `validate_exp` for Validation struct for JWT)
            // e.g. let expires_at = (now + Duration::weeks(2)).timestamp();
            let data = TokenData {
                value: user.uuid.to_urn().to_string(),
                granted_at: Utc::now().timestamp(),
                expires_at: 0,
            };
            let token = AuthorizationClaims::encode(
                data,
                &config.authorization_token_issuer,
                &config.authorization_token_key_id,
                &config.authorization_token_secret,
            );
            res.format(json!({"token": token.to_string()}))
        },
        _ => {
            warn!(logger, "signin failed: username {}", data.username);

            res.status(Status::Unauthorized).format(json!({
                "message": "The credentials you've entered are incorrect"
            }))
        },
    }
}

#[post("/logout", format = "json")]
pub fn logout(user: &User, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    // TODO
    info!(logger, "user: {}", user.uuid);

    res.status(Status::UnprocessableEntity)
}
