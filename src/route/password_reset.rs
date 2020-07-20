use chrono::{Duration, Utc};
use diesel::result::Error;
use fourche::queue::Queue;
use redis::{Commands, RedisError};
use rocket::State;
use rocket::http::{Cookies, Status};
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::job::{Job, JobKind};
use crate::model::token::{VerificationClaims, Claims, TokenData};
use crate::model::user::User;
use crate::mq::MqConn;
use crate::request::password_reset::{
    PasswordReset, PasswordResetRequest, PasswordResetUpdate,
};
use crate::request::token::verification::VerificationToken;
use crate::response::Response;
use crate::service::password_updater::PasswordUpdater;
use crate::validation::ValidationError;
use crate::validation::password_reset::Validator as PasswordResetValidator;
use crate::validation::password_reset_request::Validator as PasswordResetRequestValidator;
use crate::ss::SsConn;
use crate::util::split_token;

pub mod preflight {
    use rocket::State;
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;

    use crate::config::Config;
    use crate::response::no_content_for;

    #[options("/password/reset")]
    pub fn request<'a>(config: State<Config>) -> RawResponse<'a> {
        no_content_for("HEAD,PUT", &config)
    }

    #[options("/password/reset/<session_id>")]
    pub fn verify_update<'a>(
        session_id: String,
        config: State<Config>,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        info!(logger, "session_id: {}", session_id);
        no_content_for("GET,HEAD,PATCH", &config)
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

    #[head("/password/reset", format = "json")]
    pub fn request<'a>(
        logger: SyncLogger,
        mut cookies: Cookies,
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

    #[head("/password/reset/<session_id>", format = "json")]
    pub fn update<'a>(
        logger: SyncLogger,
        session_id: String,
        mut cookies: Cookies,
        mut ss_conn: SsConn,
    ) -> Response<'a>
    {
        info!(logger, "session_id: {}", session_id);

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

#[put("/password/reset", data = "<payload>", format = "json")]
pub fn request<'a>(
    logger: SyncLogger,
    mut cookies: Cookies,
    config: State<Config>,
    mut ss_conn: SsConn,
    mut mq_conn: MqConn,
    db_conn: DbConn,
    payload: Json<PasswordResetRequest>,
) -> Response<'a>
{
    // FIXME: create `password_renewer` service
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

    if PasswordResetRequestValidator::new(&db_conn, &payload, &logger)
        .validate()
        .is_err()
    {
        return res.status(Status::NotFound);
    }

    let email = payload.0.email;
    info!(logger, "email: {}", &email);

    if let Some(user) = User::find_by_email_only_in_available_to_reset(
        &email, &db_conn, &logger,
    ) {
        let now = Utc::now();
        let granted_at = now.timestamp();
        let expires_at = (now + Duration::hours(1)).timestamp();

        let result: Result<(i64, String), Error> = db_conn
            .build_transaction()
            .serializable()
            .deferrable()
            .read_write()
            .run::<(i64, String), diesel::result::Error, _>(|| {
                let data = TokenData {
                    value: User::generate_password_reset_token(),
                    granted_at,
                    expires_at,
                };
                let raw_token = VerificationClaims::encode(
                    data,
                    &config.verification_token_issuer,
                    &config.verification_token_key_id,
                    &config.verification_token_secret,
                );

                if let Err(e) = user.grant_token::<VerificationClaims>(
                    &raw_token,
                    &config.verification_token_issuer,
                    &config.verification_token_secret,
                    &db_conn,
                    &logger,
                ) {
                    error!(logger, "error: {}", e);
                    return Err(Error::RollbackTransaction);
                }
                Ok((user.id, raw_token))
            });

        if let Ok((id, raw_token)) = result {
            if let Some((token, sign)) = split_token(raw_token) {
                // TODO: use general value
                let session_id = User::generate_password_reset_token();
                let key = format!("pr-{}", session_id);

                // Instead of saving the signature into a cookie,
                // putting it in session store.
                //
                // Because we need to make it available users to reset password
                // also via another device than signed up, so we can't rely on
                // a cookie of http client (browser).
                let result: Result<String, RedisError> = ss_conn
                    .set_ex(&key, sign, expires_at as usize)
                    .map_err(|e| {
                        error!(logger, "error: {}", e);
                        e
                    });

                if result.is_ok() {
                    let job = Job::<String> {
                        kind: JobKind::SendPasswordResetEmail,
                        args: vec![id.to_string(), session_id, token],
                    };
                    let mut queue = Queue::new("default", &mut mq_conn);
                    if let Err(err) = queue.enqueue::<Job<String>>(job) {
                        error!(logger, "error: {}", err);
                    } else {
                        return res;
                    }
                }
            }
        }
        return res.status(Status::InternalServerError).format(json!({
            "message": "Something wrong happen, sorry :'("
        }));
    }
    res.status(Status::NotFound)
}

// TODO:
// Can't generate multiple verbs for a same route for now
// https://github.com/SergioBenitez/Rocket/issues/2
#[get("/password/reset/<session_id>", format = "json")]
pub fn verify<'a>(
    logger: SyncLogger,
    session_id: String,
    token: VerificationToken,
) -> Response<'a>
{
    info!(logger, "session_id: {}", session_id);
    info!(logger, "token: {}", &token.0);
    let res: Response = Default::default();
    res
}

// The arguments order is matter due to a spec of FromRequest
#[allow(clippy::too_many_arguments)]
#[patch("/password/reset/<session_id>", data = "<payload>", format = "json")]
pub fn update<'a>(
    logger: SyncLogger,
    mut cookies: Cookies,
    token: VerificationToken,
    config: State<Config>,
    session_id: String,
    mut ss_conn: SsConn,
    payload: Json<PasswordResetUpdate>,
    db_conn: DbConn,
) -> Response<'a>
{
    info!(logger, "session_id: {}", session_id);

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

    let mut errors: Vec<ValidationError> = vec![];
    let result = db_conn
        .build_transaction()
        .serializable()
        .deferrable()
        .run::<String, Error, _>(|| {
            match PasswordUpdater::<User>::new(&db_conn, &config, &logger)
                .load(&token)
            {
                Err(_) => Err(Error::RollbackTransaction),
                Ok(u) => {
                    let new_password = payload.0.new_password;
                    // FIXME: can we omit this clone?
                    let user = u.target.clone().unwrap();
                    let data = Json(PasswordReset {
                        username: user.username,
                        password: new_password.to_string(),
                    });
                    match PasswordResetValidator::new(&db_conn, &data, &logger)
                        .validate()
                    {
                        Err(validation_errors) => {
                            // password -> new_password
                            errors = validation_errors
                                .into_iter()
                                .filter(|v| v.field == "password")
                                .map(|mut v| {
                                    v.field = "new_password".to_string();
                                    v
                                })
                                .collect();
                            Err(Error::RollbackTransaction)
                        },
                        Ok(_) if u.update(&new_password).is_ok() => {
                            // clear session
                            let key = format!("pr-{}", session_id);
                            ss_conn
                                .del(&key)
                                .map(|r: i64| r.to_string())
                                .map_err(|e| {
                                    error!(logger, "error: {}", e);
                                    Error::RollbackTransaction
                                })
                        },
                        _ => Err(Error::RollbackTransaction),
                    }
                },
            }
        });

    match result {
        Ok(_) => res.status(Status::Ok),
        Err(_) if !errors.is_empty() => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        _ => res.status(Status::NotFound),
    }
}
