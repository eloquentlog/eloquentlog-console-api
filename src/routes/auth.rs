use std::io::{self, Read};

use regex::Regex;

use rocket::{Data, Request, Outcome::*};
use rocket::data::{FromData, Outcome, Transform, Transformed};
use rocket::http::Status;
use rocket_contrib::json::Json;
use serde_json;

use db::DbConn;
use response::Response;

use model::user::User;

// user login
const USER_LOGIN_LENGTH_LIMIT: u64 = 256;

#[derive(Debug, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

pub enum UserLoginError {
    Io(io::Error),
    Empty,
}

impl<'v> FromData<'v> for UserLogin {
    type Error = UserLoginError;
    type Owned = String;
    type Borrowed = str;

    fn transform(
        _: &Request,
        data: Data,
    ) -> Transform<Outcome<Self::Owned, Self::Error>>
    {
        let mut stream = data.open().take(USER_LOGIN_LENGTH_LIMIT);
        let mut string =
            String::with_capacity((USER_LOGIN_LENGTH_LIMIT / 2) as usize);
        let outcome = match stream.read_to_string(&mut string) {
            Ok(_) => Success(string),
            Err(e) => {
                Failure((Status::InternalServerError, UserLoginError::Io(e)))
            },
        };

        Transform::Borrowed(outcome)
    }

    fn from_data(
        _: &Request,
        outcome: Transformed<'v, Self>,
    ) -> Outcome<Self, Self::Error>
    {
        let input = outcome.borrowed()?;
        let user_login: UserLogin = match serde_json::from_str(input) {
            Ok(v) => v,
            Err(_) => {
                return Failure((
                    Status::InternalServerError,
                    UserLoginError::Empty,
                ));
            },
        };

        let username = user_login.username;
        if username == "" {
            return Failure((
                Status::UnprocessableEntity,
                UserLoginError::Empty,
            ));
        }
        // simple check as email
        if !username.contains('@') || !username.contains('.') {
            return Failure((
                Status::UnprocessableEntity,
                UserLoginError::Empty,
            ));
        }

        let password = user_login.password;
        if password == "" {
            return Failure((
                Status::UnprocessableEntity,
                UserLoginError::Empty,
            ));
        }
        // length
        if password.len() < 8 {
            return Failure((
                Status::UnprocessableEntity,
                UserLoginError::Empty,
            ));
        }
        // format
        lazy_static! {
            static ref RE: Regex = Regex::new(r"[A-z_\-\d]+").unwrap();
        }
        if !RE.is_match(&password) {
            return Failure((
                Status::UnprocessableEntity,
                UserLoginError::Empty,
            ));
        }
        Success(UserLogin { username, password })
    }
}

// actions

#[post("/login", data = "<user_login>", format = "json")]
pub fn login(
    _conn: DbConn,
    user_login: Json<UserLogin>,
) -> Result<Response, Response>
{
    let user = User {
        username: "".to_string(),
        password_hash: vec![],
    };
    if !user.verify_password(user_login.password.as_str()) {
        // TODO: error logging

        return Ok(Response {
            status: Status::Unauthorized,
            data: json!({"message": "The credentials you've entered is incorrect."}),
        });
    }

    Ok(Response {
        status: Status::Ok,
        data: json!({"message": "Success"}),
    })
}
