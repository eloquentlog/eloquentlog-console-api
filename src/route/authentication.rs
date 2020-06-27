use chrono::Utc;
use redis::{Commands, RedisError};
use rocket::State;
use rocket::http::{Cookie, Cookies, Status};
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::user::User;
use crate::model::Authenticatable;
use crate::model::token::{AuthenticationClaims, Claims, TokenData};
use crate::request::user::authentication::UserAuthentication as RequestData;
use crate::response::Response;
use crate::ss::SsConn;
use crate::util::{split_token, make_cookie};

pub mod preflight {
    use rocket::response::Response as RawResponse;
    use crate::response::no_content_for;

    #[options("/login")]
    pub fn login<'a>() -> RawResponse<'a> {
        no_content_for("HEAD,POST")
    }

    #[options("/logout")]
    pub fn logout<'a>() -> RawResponse<'a> {
        no_content_for("POST")
    }
}

pub mod preignition {
    use chrono::{Duration, Utc};
    use redis::{Commands, RedisError};
    use rocket::http::{Cookie, Cookies, SameSite, Status};
    use rocket_slog::SyncLogger;

    use crate::config::Config;
    use crate::response::Response;
    use crate::ss::SsConn;
    use crate::util::generate_random_hash;

    #[head("/login", format = "json")]
    pub fn login<'a>(
        mut cookies: Cookies,
        logger: SyncLogger,
        mut ss_conn: SsConn,
    ) -> Response<'a>
    {
        // returns CSRF token
        let res: Response = Default::default();
        info!(logger, "preignition");

        let duration = Duration::minutes(Config::CSRF_HASH_DURATION);
        let expires_at = (Utc::now() + duration).timestamp();
        let key_value = generate_random_hash(
            Config::CSRF_HASH_SOURCE,
            Config::CSRF_HASH_LENGTH,
        );
        let key = format!("xs-{}", key_value);
        let value = "1";
        let result: Result<String, RedisError> = ss_conn
            .set_ex(&key, value, expires_at as usize)
            .map_err(|e| {
                error!(logger, "error: {}", e);
                e
            });
        if result.is_ok() {
            let mut cookie = Cookie::new("csrf_token", key);
            cookie.set_http_only(true);
            cookie.set_secure(false); // TODO
            cookie.set_same_site(SameSite::Strict);
            cookies.add_private(cookie);
            return res.status(Status::Ok);
        }
        error!(logger, "something went wrong on login");
        res.status(Status::InternalServerError)
    }
}

#[post("/login", data = "<data>", format = "json")]
pub fn login<'a>(
    config: State<Config>,
    mut cookies: Cookies,
    data: RequestData,
    db_conn: DbConn,
    logger: SyncLogger,
    mut ss_conn: SsConn,
) -> Response<'a>
{
    let res: Response = Default::default();

    let cookie = cookies.get_private("csrf_token").ok_or("");
    if cookie.is_err() {
        info!(logger, "error: missing csrf_token");
        return res.status(Status::Unauthorized).format(json!({
            "message": "The CSRF token is required."
        }));
    }
    let key = cookie.ok().unwrap().value().to_string();
    let result: Result<i64, RedisError> = ss_conn.get(&key).map_err(|e| {
        error!(logger, "error: {}", e);
        e
    });
    if result.is_err() {
        return res.status(Status::Unauthorized).format(json!({
            "message": "The CSRF token has been expired. Reload the page."
        }));
    }

    match User::find_by_email(&data.username, &db_conn, &logger) {
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
                "message": "The credentials you've entered are incorrect."
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
