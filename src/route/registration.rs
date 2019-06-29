use fourche::queue::Queue;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use db::DbConn;
use job::{Job, JobKind};
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
    mq_conn: MqConn,
    logger: SyncLogger,
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
            // TODO: run within a transaction (rollback)
            let mut u = NewUser::from(&data.0);
            u.set_password(&data.password);
            if let Some(user) = User::insert(&u, &db_conn, &logger) {
                let e = NewUserEmail::from(&user);
                if let Some(email) = UserEmail::insert(&e, &db_conn, &logger) {
                    if email.grant_activation_token(&db_conn, &logger).is_ok() {
                        // send email
                        let job = Job::<i64> {
                            kind: JobKind::SendUserActivationEmail,
                            args: vec![email.id],
                        };
                        let queue = Queue::new("default", &mq_conn);
                        if let Err(err) = queue.enqueue::<Job<i64>>(job) {
                            error!(logger, "err: {}", err);
                            return res.status(Status::InternalServerError);
                        }
                    }
                    return res;
                }
            }
            // unexpected
            res.status(Status::InternalServerError)
        },
    }
}

#[post("/deregister", format = "json")]
pub fn deregister(user: User, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    // TODO
    info!(logger, "user: {}", user.uuid);

    res.status(Status::UnprocessableEntity)
}
