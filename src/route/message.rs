use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use crate::db::DbConn;
use crate::model::message::{Message, NewMessage};
use crate::model::user::User;
use crate::response::Response;
use crate::request::message::Message as RequestData;
use crate::validation::message::Validator;

const MESSAGES_PER_REQUEST: i64 = 100;

pub mod preflight {
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;

    use crate::response::no_content_for;

    #[options("/message/append/<key>")]
    pub fn append<'a>(key: String, logger: SyncLogger) -> RawResponse<'a> {
        info!(logger, "key: {}", key);
        no_content_for("POST")
    }

    #[options("/message/lrange/<key>/<start>/<stop>")]
    pub fn lrange<'a>(
        key: String,
        start: i64,
        stop: i64,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        info!(logger, "key: {}, start: {}, stop: {}", key, start, stop);
        no_content_for("GET")
    }
}

// Save a new log message.
//
// The value looks like this:
//
// ```json
// {
//    "title": "",
//    "level": 0,
//    "content": ""
//    ...
// }
// ```
#[post("/message/append/<key>", format = "json", data = "<data>")]
pub fn append(
    user: &User,
    key: String,
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
) -> Response
{
    let res: Response = Default::default();

    // TODO
    info!(logger, "user: {}", user.uuid);
    info!(logger, "key: {}", key);

    let v = Validator::new(&data, &logger);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            let m = NewMessage::from(data.0.clone());
            if let Some(id) = Message::insert(&m, &conn, &logger) {
                info!(logger, "user: {}", user.uuid);
                return res.format(json!({"message": {
                    "id": id,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}

#[get("/message/lrange/<key>/<start>/<stop>")]
pub fn lrange(
    user: &User,
    key: String,
    start: i64,
    stop: i64,
    conn: DbConn,
    logger: SyncLogger,
) -> Response
{
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);
    info!(logger, "key: {}, start: {}, stop: {}", key, start, stop);

    // TODO
    let mut offset = start;
    if offset < 1 {
        offset = 0;
    }

    let mut limit = stop - start + 2;
    if limit < 1 {
        limit = 1;
    }

    // FIXME: by uuid
    let stream_id = 1;
    let data = match Message::fetch_messages_by_stream_id(
        key, stream_id, offset, limit, &conn, &logger,
    ) {
        None => {
            error!(logger, "err: not found user.id {}", user.uuid);
            vec![]
        },
        Some(a) => a.iter().map(|m| json!({ "message": m })).collect(),
    };
    res.format(json!(data))
}
