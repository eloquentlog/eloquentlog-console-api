use rocket::State;
use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::token::AuthorizationClaims;
use model::user::{NewUser, User};
use response::Response;
use request::auth::AuthorizationToken;
use request::user::User as RequestData;
use validation::user::Validator;

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
            let mut new_user = NewUser::from(data.0.clone());
            new_user.set_password(&data.password);
            if let Some(u) = User::insert(&new_user, &conn, &logger) {
                // TODO: run it in worker
                if let Some(user) = u.grant_activation_token(
                    &config.jwt_issuer,
                    &config.jwt_key_id,
                    &config.jwt_secret,
                    &conn,
                    &logger,
                ) {
                    // TODO
                    // send mail
                    info!(
                        logger,
                        "activation_token: {}",
                        user.activation_token.unwrap(),
                    );
                    return res;
                }
            }
            res.status(Status::InternalServerError)
        },
    }
}

#[get("/activate/<activation_token>", format = "json")]
pub fn activate(
    activation_token: String,
    _conn: DbConn,
    logger: SyncLogger,
    _config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    info!(logger, "activation_token: {}", activation_token);
    res.status(Status::Ok)
}

#[post("/deregister", format = "json")]
pub fn deregister(
    token: AuthorizationToken,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    let user = User::find_by_token::<AuthorizationClaims>(
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
