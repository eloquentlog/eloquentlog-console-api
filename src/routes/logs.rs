use rocket::http::Status;
use rocket_contrib::json::Json;
use response::Response;
use request::Message;

#[get("/logs")]
pub fn get() -> Response {
    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({"message": []}))
}

#[post("/logs", format = "json", data = "<message>")]
pub fn post(message: Json<Message>) -> Response {
    // TODO
    println!("title: {}", message.0.title);
    println!("description: {}", message.0.description.unwrap_or_default());

    let res = Response {
        status: Status::Ok,
        data: json!(null),
    };
    res.format(json!({"message": {
        "id": 0,
    }}))
}

#[put("/logs/<id>", format = "json", data = "<message>")]
pub fn put(id: usize, message: Json<Message>) -> Response {
    // TODO
    println!("id: {}", id);

    let message_id = message.0.id.unwrap_or_default();
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
        "id": id,
    }}))
}
