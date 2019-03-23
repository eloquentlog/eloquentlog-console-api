use rocket::http::Status;
use rocket_contrib::json::Json;

use db::DbConn;
use model::message::{Format, Level, Message, NewMessage};
use response::Response;
use request::Message as RequestData;
use validation::message::Validator;

const MESSAGES_PER_REQUEST: i64 = 100;

#[get("/messages")]
pub fn get(conn: DbConn) -> Response {
    let res: Response = Default::default();

    let messages = Message::recent(MESSAGES_PER_REQUEST, &conn);
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
pub fn post(data: Json<RequestData>, conn: DbConn) -> Response {
    let res: Response = Default::default();

    let v = Validator::new(&data);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            // TODO
            let m = NewMessage::from(data.0.clone());
            if let Some(id) = Message::insert(&m, &conn) {
                return res.format(json!({"message": {
                    "id": id,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}

#[put("/messages/<id>", format = "json", data = "<data>")]
pub fn put(id: usize, data: Json<RequestData>, conn: DbConn) -> Response {
    let res: Response = Default::default();

    let message_id = data.0.id.unwrap_or_default();
    if message_id == 0 || id != message_id {
        return res.status(Status::NotFound).format(json!(null));
    }

    let result = Message::first(id as i64, &conn);
    if result.is_none() {
        return res.status(Status::NotFound).format(json!(null));
    }

    let v = Validator::new(&data);
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
            m.level = Level::from(
                data.level.unwrap_or_else(|| "information".to_string()),
            );
            m.format =
                Format::from(data.format.unwrap_or_else(|| "toml".to_string()));
            m.title = data.title.unwrap_or_default();
            m.content = data.content;

            if let Some(id) = Message::update(&mut m, &conn) {
                return res.format(json!({"message": {
                    "id": id,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}
