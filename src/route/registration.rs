use rocket::http::Status;
use rocket_contrib::json::Json;

use db::DbConn;
use model::user::{NewUser, User};
use response::Response;
use request::User as RequestData;
use validation::user::Validator;

#[post("/register", format = "json", data = "<data>")]
pub fn register(data: Json<RequestData>, conn: DbConn) -> Response {
    let res: Response = Default::default();

    let v = Validator::new(&data);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            let mut u = NewUser::from(data.0.clone());
            u.set_password(&data.password);
            if let Some(id) = User::insert(&u, &conn) {
                return res.format(json!({"user": {
                    "id": id,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}

#[post("/deregister", format = "json")]
pub fn deregister(_conn: DbConn) -> Response {
    // TODO
    let res: Response = Default::default();
    res.status(Status::UnprocessableEntity)
}
