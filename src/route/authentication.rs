use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use db::DbConn;
use model::user::User;
use request::UserLogin as RequestData;
use response::Response;

#[post("/login", data = "<data>", format = "json")]
pub fn login(
    conn: DbConn,
    data: Json<RequestData>,
    logger: SyncLogger,
) -> Response
{
    let res: Response = Default::default();

    let user_login = data.0.clone();

    match User::find_by_email_or_username(&user_login.username, &conn) {
        Some(user) => {
            if !user.verify_password(&user_login.password) {
                error!(logger, "login failed: user_id {}", user.id);

                return res.status(Status::Unauthorized).format(json!({
                    "message": "The credentials you've entered is incorrect."
                }));
            }
            res.format(json!({"message": "Success"}))
        },
        None => res.status(Status::Unauthorized),
    }
}

#[post("/logout", format = "json")]
pub fn logout(_conn: DbConn) -> Response {
    // TODO
    let res: Response = Default::default();
    res.status(Status::UnprocessableEntity)
}
