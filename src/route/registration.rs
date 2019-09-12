use chrono::{Duration, Utc};
use diesel::result::Error;
use fourche::queue::Queue;
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

                    // TODO:
                    // impl service object handles token generation/activation.
                    // see also signin
                    let now = Utc::now();
                    let data = TokenData {
                        value: UserEmail::generate_token(),
                        granted_at: now.timestamp(),
                        expires_at: (now + Duration::hours(1)).timestamp(),
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
                // TODO:
                // This enforces user to use same browser also on activation
                // because of cookie. Consider another mechanims.
                // Use raw activation token? (+ session token)
                if let Some((payload, signature)) = split_token(token) {
                    let job = Job::<String> {
                        kind: JobKind::SendUserActivationEmail,
                        args: vec![id.to_string(), payload],
                    };
                    // TODO: Consider about retrying
                    let mut queue = Queue::new("default", &mut mq_conn);
                    if let Err(err) = queue.enqueue::<Job<String>>(job) {
                        error!(logger, "error: {}", err);
                    } else {
                        return res.cookies(vec![signature]);
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
