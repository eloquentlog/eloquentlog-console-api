use rocket::State;
use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::user::{NewUser, User};
use response::Response;
use request::auth::AuthToken;
use request::user::User as RequestData;
use validation::user::Validator;

#[post("/register", format = "json", data = "<data>")]
pub fn register(
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
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
            let mut u = NewUser::from(data.0.clone());
            u.set_password(&data.password);
            if let Some(id) = User::insert(&u, &conn, &logger) {
                return res.format(json!({"user": {
                    "id": id,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}

#[post("/deregister", format = "json")]
pub fn deregister(
    token: AuthToken,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    let user = User::find_by_jwt(
        &token,
        &config.jwt_issuer,
        &config.jwt_secret,
        &conn,
        &logger,
    )
    .unwrap();

    // TODO
    info!(logger, "deregister: {}", user.id);

    res.status(Status::UnprocessableEntity)
}
