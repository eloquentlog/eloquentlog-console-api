use std::any::Any;

use rocket::http::Status;
use rocket_contrib::json::Json;

use db::DbConn;
use model::message::{Message, NewMessage};
use response::Response;
use request::Message as Data;
use validation::message::Validator;

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
    let res: Response = Default::default();

    let v = Validator::new(data);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(v) => {
            if let Ok(m) = (v as Box<Any>).downcast::<NewMessage>() {
                if let Some(id) = Message::insert(&m, &conn) {
                    return res.format(json!({"message": {
                        "id": id,
                    }}));
                }
            }
            res.status(Status::InternalServerError)
        },
    }
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
