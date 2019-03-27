use rocket::http::Status;
use rocket_contrib::json::Json;

use db::DbConn;
use model::user::User;
use request::UserLogin as RequestData;
use response::Response;

#[post("/login", data = "<data>", format = "json")]
pub fn login(conn: DbConn, data: Json<RequestData>) -> Response {
    let res: Response = Default::default();

    let user_login = data.0.clone();

    match User::find_by_email_or_username(&user_login.username, &conn) {
        Some(user) => {
            if !user.verify_password(&user_login.password) {
                // TODO: error logging

                return res.status(Status::Unauthorized).format(json!({
                    "message": "The credentials you've entered is incorrect."
                }));
            }
            res.format(json!({"message": "Success"}))
        },
        None => res.status(Status::Unauthorized),
    }
}
