use chrono::{Duration, Utc};
use diesel::result::Error;
use fourche::queue::Queue;
use redis::{Commands, RedisError};
use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use job::{Job, JobKind};
use model::token::{ActivationClaims, Claims, TokenData};
use model::user::{NewUser, User};
use model::user_email::{NewUserEmail, UserEmail};
use response::{Response, no_content_for};
use request::user::UserSignUp as RequestData;
use mq::MqConn;
use validation::user::Validator;
use ss::SsConn;
use util::split_token;

#[options("/register")]
pub fn register_options<'a>() -> RawResponse<'a> {
    no_content_for("POST")
}

#[post("/register", format = "json", data = "<data>")]
pub fn register<'a>(
    data: Json<RequestData>,
    db_conn: DbConn,
    mut mq_conn: MqConn,
    mut ss_conn: SsConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response<'a>
{
    let res: Response = Default::default();

    let v = Validator::new(&db_conn, &data, &logger);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            // TODO:
            // impl service object handles token generation/activation.
            // see also signin
            let now = Utc::now();
            let granted_at = now.timestamp();
            let expires_at = (now + Duration::hours(1)).timestamp();

            let result: Result<(i64, String), Error> = db_conn
                .build_transaction()
                .serializable()
                .deferrable()
                .read_write()
                .run::<(i64, String), diesel::result::Error, _>(|| {
                    let mut u = NewUser::from(&data.0);
                    u.set_password(&data.password);
                    let user = User::insert(&u, &db_conn, &logger).unwrap();
                    let ue = NewUserEmail::from(&user);
                    let user_email =
                        UserEmail::insert(&ue, &db_conn, &logger).unwrap();

                    let data = TokenData {
                        value: UserEmail::generate_token(),
                        granted_at,
                        expires_at,
                    };
                    let token = ActivationClaims::encode(
                        data,
                        &config.activation_token_issuer,
                        &config.activation_token_key_id,
                        &config.activation_token_secret,
                    );

                    if let Err(e) = user_email.grant_token::<ActivationClaims>(
                        &token,
                        &config.activation_token_issuer,
                        &config.activation_token_secret,
                        &db_conn,
                        &logger,
                    ) {
                        error!(logger, "error: {}", e);
                        return Err(Error::RollbackTransaction);
                    }
                    Ok((user_email.id, token))
                });

            if let Ok((id, token)) = result {
                if let Some((payload, signature)) = split_token(token) {
                    // Instead of saving the signature into a cookie,
                    // putting it in session store.
                    //
                    // Because we need to make it available users to activate
                    // the account also via another device than signed up, so
                    // we can't rely on a cookie of http client (browser).
                    let rawsig = signature.value();
                    let sessid = UserEmail::generate_token();

                    // TODO: Async with tokio? (consider about retrying)
                    let result: Result<String, RedisError> = ss_conn
                        .set_ex(&sessid, rawsig, expires_at as usize)
                        .map_err(|e| {
                            error!(logger, "error: {}", e);
                            e
                        });

                    if result.is_ok() {
                        let job = Job::<String> {
                            kind: JobKind::SendUserActivationEmail,
                            args: vec![id.to_string(), payload, sessid],
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
            res.status(Status::InternalServerError).format(json!({
                "message": "Something wrong happen, sorry :'("
            }))
        },
    }
}

#[post("/deregister", format = "json")]
pub fn deregister(user: &User, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    // TODO
    info!(logger, "user: {}", user.uuid);

    res.status(Status::UnprocessableEntity)
}
