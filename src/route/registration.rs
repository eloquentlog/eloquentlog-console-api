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

#[options("/register")]
pub fn register_options<'a>() -> RawResponse<'a> {
    no_content_for("POST")
}

#[post("/register", format = "json", data = "<data>")]
pub fn register(
    data: Json<RequestData>,
    db_conn: DbConn,
    mut mq_conn: MqConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
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
            let result: Result<UserEmail, Error> = db_conn
                .build_transaction()
                .serializable()
                .deferrable()
                .read_write()
                .run::<UserEmail, diesel::result::Error, _>(|| {
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
                    Ok(user_email)
                });

            match result {
                Ok(user_email) => {
                    let job = Job::<i64> {
                        kind: JobKind::SendUserActivationEmail,
                        args: vec![user_email.id],
                    };
                    // TODO: Consider about retrying
                    let mut queue = Queue::new("default", &mut mq_conn);
                    if let Err(err) = queue.enqueue::<Job<i64>>(job) {
                        error!(logger, "error: {}", err);
                    }
                    res
                },
                _ => res.status(Status::InternalServerError),
            }
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
