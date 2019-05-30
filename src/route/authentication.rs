use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::token::AuthorizationClaims;
use model::user::User;
use request::auth::AuthorizationToken;
use request::user::UserLogin as RequestData;
use response::Response;

#[post("/login", data = "<data>", format = "json")]
pub fn login(
    conn: DbConn,
    data: RequestData,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    match User::find_by_email_or_uuid(&data.username, &conn, &logger) {
        Some(ref user) if user.verify_password(&data.password) => {
            // TODO
            let token = user.generate_authorization_token(
                &config.authorization_token_key_id,
                &config.authorization_token_issuer,
                &config.authorization_token_secret,
            );
            res.format(json!({"token": token.to_string()}))
        },
        _ => {
            warn!(logger, "login failed: username {}", data.username);

            res.status(Status::Unauthorized).format(json!({
                "message": "The credentials you've entered is incorrect"
            }))
        },
    }
}

#[post("/logout", format = "json")]
pub fn logout(
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
    info!(logger, "logout: {}", user.id);

    res.status(Status::UnprocessableEntity)
}
