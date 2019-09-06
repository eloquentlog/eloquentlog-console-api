use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use request::user::UserActivation as RequestData;
use response::{Response, no_content_for};
use service::activator::UserActivator;

#[options("/user/activate")]
pub fn activate_options<'a>() -> RawResponse<'a> {
    no_content_for("PUT")
}

#[put("/user/activate", data = "<data>", format = "json")]
pub fn activate(
    data: RequestData,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    let activator = UserActivator::new(&conn, &config, &logger);
    if activator.activate(&data.token).is_ok() {
        res.status(Status::Ok)
    } else {
        res.status(Status::BadRequest).format(json!({
            "message": "The token has been expired or is invalid"
        }))
    }
}
