use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::user::User;
use request::user::UserSignIn as RequestData;
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
            let voucher = user.generate_authorization_voucher(
                &config.authorization_voucher_key_id,
                &config.authorization_voucher_issuer,
                &config.authorization_voucher_secret,
            );
            res.format(json!({"voucher": voucher.to_string()}))
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
pub fn logout(user: User, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    // TODO
    info!(logger, "user: {}", user.uuid);

    res.status(Status::UnprocessableEntity)
}
