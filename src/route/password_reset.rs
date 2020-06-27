use chrono::{Duration, Utc};
use diesel::result::Error;
use fourche::queue::Queue;
use redis::{Commands, RedisError};
use rocket::State;
use rocket::http::Status;
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
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;
    use crate::response::no_content_for;

    #[options("/password/reset")]
    pub fn request<'a>() -> RawResponse<'a> {
        no_content_for("PUT")
    }

    #[options("/password/reset/<session_id>")]
    pub fn verify_update<'a>(
        session_id: String,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        info!(logger, "session_id: {}", session_id);
        no_content_for("GET,PATCH")
    }
}

#[put("/password/reset", data = "<payload>", format = "json")]
pub fn request<'a>(
    payload: Json<PasswordResetRequest>,
    db_conn: DbConn,
    mut mq_conn: MqConn,
    mut ss_conn: SsConn,
    config: State<Config>,
    logger: SyncLogger,
) -> Response<'a>
{
    // FIXME: create `password_renewer` service
    let res: Response = Default::default();

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
    session_id: String,
    token: VerificationToken,
    logger: SyncLogger,
) -> Response<'a>
{
    info!(logger, "session_id: {}", session_id);
    info!(logger, "token: {}", &token.0);
    let res: Response = Default::default();
    res
}

#[patch("/password/reset/<session_id>", data = "<payload>", format = "json")]
pub fn update<'a>(
    session_id: String,
    token: VerificationToken,
    payload: Json<PasswordResetUpdate>,
    db_conn: DbConn,
    mut ss_conn: SsConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response<'a>
{
    info!(logger, "session_id: {}", session_id);

    let res: Response = Default::default();

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
