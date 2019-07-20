use diesel::result::Error;
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
            let result: Result<(), Error> = db_conn
                .build_transaction()
                .serializable()
                .deferrable()
                .read_write()
                .run(|| {
                    let mut u = NewUser::from(&data.0);
                    u.set_password(&data.password);
                    let user = User::insert(&u, &db_conn, &logger).unwrap();
                    let ue = NewUserEmail::from(&user);
                    let user_email =
                        UserEmail::insert(&ue, &db_conn, &logger).unwrap();
                    if let Err(e) =
                        user_email.grant_activation_token(&db_conn, &logger)
                    {
                        error!(logger, "error: {}", e);
                        return Err(Error::RollbackTransaction);
                    }
                    // send email
                    let job = Job::<i64> {
                        kind: JobKind::SendUserActivationEmail,
                        args: vec![user_email.id],
                    };
                    let queue = Queue::new("default", &mq_conn);
                    if let Err(err) = queue.enqueue::<Job<i64>>(job) {
                        error!(logger, "error: {}", err);
                        return Err(Error::RollbackTransaction);
                    }
                    Ok(())
                });

            // unexpected
            if result.is_err() {
                return res.status(Status::InternalServerError);
            }
            res
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
