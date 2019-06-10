use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::user::{NewUser, User};
use model::user_email::{NewUserEmail, UserEmail};
use response::{Response, no_content_for};
use request::user::UserSignUp as RequestData;
use validation::user::Validator;

#[options("/register")]
pub fn register_options<'a>() -> RawResponse<'a> {
    no_content_for("POST")
}

#[post("/register", format = "json", data = "<data>")]
pub fn register(
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    let v = Validator::new(&conn, &data, &logger);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            // TODO:
            // * run within a transaction
            // * run it in worker
            let mut u = NewUser::from(&data.0);
            u.set_password(&data.password);
            if let Some(user) = User::insert(&u, &conn, &logger) {
                let e = NewUserEmail::from(&user);
                if let Some(email) = UserEmail::insert(&e, &conn, &logger) {
                    // This updates created user_email
                    let voucher = email
                        .grant_activation_voucher(
                            &config.activation_voucher_issuer,
                            &config.activation_voucher_key_id,
                            &config.activation_voucher_secret,
                            &conn,
                            &logger,
                        )
                        .unwrap();

                    // TODO: send email
                    info!(logger, "activation_voucher: {}", voucher);
                    return res;
                }
            }
            res.status(Status::InternalServerError)
        },
    }
}

#[get("/activate/<voucher>", format = "json")]
pub fn activate(voucher: String, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    // TODO
    info!(logger, "voucher: {}", voucher);

    res.status(Status::Ok)
}

#[post("/deregister", format = "json")]
pub fn deregister(user: User, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    // TODO
    info!(logger, "user: {}", user.uuid);

    res.status(Status::UnprocessableEntity)
}
