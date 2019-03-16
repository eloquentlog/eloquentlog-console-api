use rocket::http::Status;
use rocket_contrib::json::Json;
use response::Response;
use request::Message as Data;

use db::DbConn;
use model::message::{Format, Level, Message, NewMessage};

#[get("/messages")]
pub fn get() -> Response {
    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({"messages": []}))
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
pub fn post(data: Json<Data>, conn: DbConn) -> Response {
    // TODO
    // * validations
    // * default values
    let m = NewMessage {
        code: data.0.code.unwrap_or_default(),
        lang: "en".to_string(),
        level: Level::Information,
        format: Format::TOML,
        title: data.0.title,
        content: data.0.content.unwrap_or_default(),
    };

    let inserted = Message::insert(&m, &conn);
    println!("inserted: {}", inserted);

    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({"message": {
        "id": 0,
    }}))
}

#[put("/messages/<id>", format = "json", data = "<data>")]
pub fn put(id: usize, data: Json<Data>) -> Response {
    // TODO
    println!("id: {}", id);

    let message_id = data.0.id.unwrap_or_default();
    if message_id == 0 || id != message_id {
        return Response {
            status: Status::NotFound,
            data: json!(null),
        };
    }

    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({"message": {
        "id": message_id,
    }}))
}
