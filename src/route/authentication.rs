use chrono::Utc;
use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::user::User;
use model::token::{AuthenticationClaims, Claims, TokenData};
use request::user::authentication::UserAuthentication as RequestData;
use response::{Response, no_content_for};
use util::split_token;

#[options("/login")]
pub fn login_options<'a>() -> RawResponse<'a> {
    no_content_for("POST")
}

#[post("/login", data = "<data>", format = "json")]
pub fn login<'a>(
    data: RequestData,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response<'a>
{
    let res: Response = Default::default();

    match User::find_by_email(&data.username, &conn, &logger) {
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
            let authentication_token = AuthenticationClaims::encode(
                data,
                &config.authentication_token_issuer,
                &config.authentication_token_key_id,
                &config.authentication_token_secret,
            );

            // TODO:
            // * consider about implementation "Are you there?" modal
            // * consider about extension (re-set it again?)
            let (token, sign) = match split_token(authentication_token) {
                Some(result) => result,
                None => {
                    return res.status(Status::InternalServerError).format(
                        json!({
                         "message": "Something wrong happen, sorry :'("
                        }),
                    );
                },
            };

            res.cookies(vec![sign]).format(json!({ "token": token }))
        },
        _ => {
            warn!(logger, "login failed: username {}", data.username);

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
