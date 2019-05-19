use rocket::http::Status;
use rocket_slog::SyncLogger;

use db::DbConn;
use model::user::User;
use request::UserLogin as RequestData;
use response::Response;

#[post("/login", data = "<data>", format = "json")]
pub fn login(conn: DbConn, data: RequestData, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    match User::find_by_email_or_username(&data.username, &conn) {
        Some(ref user) if user.verify_password(&data.password) => {
            res.format(json!({ "message": "Success" }))
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
