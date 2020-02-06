use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use crate::db::DbConn;
use crate::model::message::{LogFormat, LogLevel, Message, NewMessage};
use crate::model::user::User;
use crate::response::Response;
use crate::request::message::Message as RequestData;
use crate::validation::message::Validator;

const MESSAGES_PER_REQUEST: i64 = 100;

pub mod preflight {
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;
    use crate::response::no_content_for;

    // TODO: refer interfaces for Redis
    #[options("/message/lrange")]
    pub fn lrange<'a>() -> RawResponse<'a> {
        no_content_for("GET")
    }

    #[options("/message/add")]
    pub fn add<'a>() -> RawResponse<'a> {
        no_content_for("POST")
    }

    #[options("/message/put/<id>")]
    pub fn put<'a>(id: usize, logger: SyncLogger) -> RawResponse<'a> {
        info!(logger, "id: {}", id);
        no_content_for("PUT")
    }
}

// Save a new log message.
//
// ```json
// {
//    "title": "",
//    "level": 0,
//    "content": ""
//    ...
// }
// ```
// FIXME: rename function
#[post("/message/add", format = "json", data = "<data>")]
pub fn add(
    user: &User,
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
) -> Response
{
    let res: Response = Default::default();

    let v = Validator::new(&data, &logger);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            let mut m = NewMessage::from(data.0.clone());
            m.user_id = user.id;
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

// FIXME: take args
#[get("/message/lrange")]
pub fn lrange(user: &User, conn: DbConn, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);

    let messages = Message::recent_by_user_id(
        user.id,
        MESSAGES_PER_REQUEST,
        &conn,
        &logger,
    );
    res.format(json!({ "message": messages }))
}

// FIXME: rename function
#[put("/message/put/<id>", format = "json", data = "<data>")]
pub fn put(
    user: &User,
    id: usize,
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
) -> Response
{
    let res: Response = Default::default();

    let message_id = data.0.id.unwrap_or_default();
    if message_id == 0 || id != message_id {
        info!(logger, "user: {}, message_id: {}", user.uuid, message_id);
        return res.status(Status::NotFound).format(json!(null));
    }

    let result = Message::first_by_user_id(id as i64, user.id, &conn, &logger);
    if result.is_none() {
        return res.status(Status::NotFound).format(json!(null));
    }

    let v = Validator::new(&data, &logger);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            // TODO: refactor construction
            let data = data.0.clone();
            let mut m = result.unwrap();
            m.code = data.code;
            m.lang = data.lang.unwrap_or_default();
            m.level = LogLevel::from(
                data.level.unwrap_or_else(|| "information".to_string()),
            );
            m.format = LogFormat::from(
                data.format.unwrap_or_else(|| "toml".to_string()),
            );
            m.title = data.title.unwrap_or_default();
            m.content = data.content;

            if let Some(id) = Message::update(&mut m, &conn, &logger) {
                return res.format(json!({"message": {
                    "id": id,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}
