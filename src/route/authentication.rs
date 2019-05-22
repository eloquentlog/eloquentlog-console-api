use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::user::User;
use request::UserLogin as RequestData;
use response::Response;

#[post("/login", data = "<data>", format = "json")]
pub fn login(
    conn: DbConn,
    data: RequestData,
    logger: SyncLogger,
    state: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    match User::find_by_email_or_username(&data.username, &conn) {
        Some(ref user) if user.verify_password(&data.password) => {
            // TODO
            let token = user.to_jwt(
                &state.jwt_key_id,
                &state.jwt_issuer,
                &state.jwt_secret,
            );
            res.format(json!({ "message": token.to_string() }))
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
pub fn logout(_conn: DbConn) -> Response {
    // TODO
    let res: Response = Default::default();
    res.status(Status::UnprocessableEntity)
}
