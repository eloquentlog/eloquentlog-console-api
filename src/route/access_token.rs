use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use crate::db::DbConn;
use crate::model::access_token::{AccessToken};
use crate::model::user::User;
use crate::response::{Response, no_content_for};

#[options("/access_token")]
pub fn get_access_token_preflight<'a>() -> RawResponse<'a> {
    no_content_for("GET")
}

#[get("/access_token")]
pub fn get_access_token(
    user: &User,
    conn: DbConn,
    logger: SyncLogger,
) -> Response
{
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);

    match AccessToken::find_personal_token_by_user_id(user.id, &conn, &logger) {
        None => {
            error!(logger, "err: not found user.id {}", user.uuid);
            res.status(Status::NotFound)
        },
        Some(t) => {
            let token = if t.revoked_at.is_some() {
                "".to_string()
            } else {
                String::from_utf8(t.token.unwrap()).unwrap()
            };
            res.format(json!({"access_token": {
                "name": t.name,
                "token": token,
                "revoked_at": t.revoked_at,
                "created_at": t.created_at,
                "updated_at": t.updated_at,
            }}))
        },
    }
}
