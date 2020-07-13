use rocket_slog::SyncLogger;

use crate::db::DbConn;
use crate::model::namespace::Namespace;
use crate::model::user::User;
use crate::response::Response;

pub mod preflight {
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;

    use crate::response::no_content_for;

    #[options("/namespace/hgetall")]
    pub fn hgetall<'a>(logger: SyncLogger) -> RawResponse<'a> {
        info!(logger, "hgetall");
        no_content_for("GET")
    }
}

#[get("/namespace/hgetall")]
pub fn hgetall(user: &User, conn: DbConn, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);

    let data = match Namespace::find_all(user, &conn, &logger) {
        None => {
            error!(logger, "err: no namespace for user: {}", user.uuid);
            vec![]
        },
        Some(a) => a.iter().map(|n| json!({ "namespace": n })).collect(),
    };
    res.format(json!(data))
}
