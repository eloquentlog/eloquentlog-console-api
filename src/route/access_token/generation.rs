use chrono::Utc;
use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::access_token::{AccessToken};
use crate::model::token::{AuthenticationClaims, Claims, TokenData};
use crate::model::user::User;
use crate::response::{Response, no_content_for};

#[options("/access_token/generate")]
pub fn generate_preflight<'a>() -> RawResponse<'a> {
    no_content_for("GET")
}

#[get("/access_token/generate")]
pub fn generate<'a>(
    user: &User,
    conn: DbConn,
    config: State<Config>,
    logger: SyncLogger,
) -> Response<'a>
{
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);

    match AccessToken::find_personal_token_by_user_id(user.id, &conn, &logger) {
        None => {
            error!(logger, "err: not found user.id {}", user.uuid);
            res.status(Status::NotFound)
        },
        Some(t) => {
            let value = if t.revoked_at.is_some() {
                "".to_string()
            } else {
                String::from_utf8(t.token.unwrap()).unwrap()
            };

            // this value is not saved
            let data = TokenData {
                value,
                granted_at: Utc::now().timestamp(),
                expires_at: 0,
            };
            let token = AuthenticationClaims::encode(
                data,
                &config.authentication_token_issuer,
                &config.authentication_token_key_id,
                &config.authentication_token_secret,
            );

            res.format(json!({"access_token": {
                "name": t.name,
                "token": token,
                "revoked_at": t.revoked_at,
                "created_at": t.created_at,
                "updated_at": t.updated_at,
            }}))
        },
    }
}
