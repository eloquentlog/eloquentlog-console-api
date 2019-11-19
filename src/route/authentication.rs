use chrono::Utc;
use rocket::State;
use rocket::http::{Cookie, Cookies, Status};
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::user::User;
use crate::model::Authenticatable;
use crate::model::token::{AuthenticationClaims, Claims, TokenData};
use crate::request::user::authentication::UserAuthentication as RequestData;
use crate::response::{Response, no_content_for};
use crate::util::{split_token, make_cookie};

#[options("/login")]
pub fn login_preflight<'a>() -> RawResponse<'a> {
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

            let cookie = make_cookie(sign);
            res.cookies(vec![cookie]).format(json!({ "token": token }))
        },
        _ => {
            warn!(logger, "login failed: username {}", data.username);

            res.status(Status::Unauthorized).format(json!({
                "message": "The credentials you've entered are incorrect"
            }))
        },
    }
}

// logout
//
// * Remove a cookie
// * Delete session value in Redis
#[post("/logout", format = "json")]
pub fn logout<'a>(
    mut cookies: Cookies,
    user: &User,
    logger: SyncLogger,
) -> Response<'a>
{
    let res: Response = Default::default();
    info!(logger, "user: {}", user.uuid);

    // TODO: remove_private
    cookies.remove(Cookie::named("sign"));

    res.status(Status::Ok)
}
