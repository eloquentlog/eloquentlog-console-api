use rocket::State;
use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::message::{LogFormat, LogLevel, Message, NewMessage};
use model::user::User;
use response::Response;
use request::auth::AuthToken;
use request::message::Message as RequestData;
use validation::message::Validator;

const MESSAGES_PER_REQUEST: i64 = 100;

#[get("/messages")]
pub fn get(
    token: AuthToken,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    // TODO: fetch messages for the user
    let _ = User::find_by_jwt(
        &token,
        &config.jwt_issuer,
        &config.jwt_secret,
        &conn,
        &logger,
    )
    .unwrap();

    let messages = Message::recent(MESSAGES_PER_REQUEST, &conn, &logger);
    res.format(json!({ "messages": messages }))
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
#[post("/messages", format = "json", data = "<data>")]
pub fn post(
    token: AuthToken,
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    // TODO: save messages for the user
    let _ = User::find_by_jwt(
        &token,
        &config.jwt_issuer,
        &config.jwt_secret,
        &conn,
        &logger,
    )
    .unwrap();

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
                return res.format(json!({"message": {
                    "id": id,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}

#[put("/messages/<id>", format = "json", data = "<data>")]
pub fn put(
    token: AuthToken,
    id: usize,
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    // TODO: update messages for the user
    let _ = User::find_by_jwt(
        &token,
        &config.jwt_issuer,
        &config.jwt_secret,
        &conn,
        &logger,
    )
    .unwrap();

    let message_id = data.0.id.unwrap_or_default();
    if message_id == 0 || id != message_id {
        info!(logger, "message_id: {}", message_id);
        return res.status(Status::NotFound).format(json!(null));
    }

    let result = Message::first(id as i64, &conn, &logger);
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
            // TODO
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
