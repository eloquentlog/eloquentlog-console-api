use rocket::State;
use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::token::AuthorizationClaims;
use model::user::{NewUser, User};
use model::user_email::{NewUserEmail, UserEmail};
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
            // TODO: run within a transaction
            let mut u = NewUser::from(data.0.clone());
            u.set_password(&data.password);
            if let Some(user) = User::insert(&u, &conn, &logger) {
                // TODO: run it in worker
                let e = NewUserEmail::from(user);
                // FIXME: reduce queries
                if let Some(e) = UserEmail::insert(&e, &conn, &logger) {
                    if let Some(user_email) = e.grant_activation_token(
                        &config.authorization_token_issuer,
                        &config.authorization_token_key_id,
                        &config.authorization_token_secret,
                        &conn,
                        &logger,
                    ) {
                        // TODO: send email
                        info!(
                            logger,
                            "activation_token: {}",
                            user_email.activation_token.unwrap(),
                        );
                        return res;
                    }
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
        &config.authorization_token_issuer,
        &config.authorization_token_secret,
        &conn,
        &logger,
    )
    .unwrap();

    // TODO
    info!(logger, "deregister: {}", user.id);

    res.status(Status::UnprocessableEntity)
}
