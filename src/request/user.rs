use std::io::{self, Read};

use rocket::{Data, Request, Outcome::*};
use rocket::data::{self, FromData, Transform, Transformed};
use rocket::http::Status;
use serde_json;

/// User
#[derive(Clone, Deserialize)]
pub struct User {
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub password: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            name: None,
            username: None,
            email: "".to_string(),
            password: "".to_string(),
        }
    }
}

/// UserLogin
pub enum UserLoginError {
    Io(io::Error),
    Empty,
}

const USER_LOGIN_LENGTH_LIMIT: u64 = 256;

#[derive(Clone, Debug, Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

impl<'v> FromData<'v> for UserLogin {
    type Error = UserLoginError;
    type Owned = String;
    type Borrowed = str;

    fn transform(
        _: &Request,
        data: Data,
    ) -> Transform<data::Outcome<Self::Owned, Self::Error>>
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
    ) -> data::Outcome<Self, Self::Error>
    {
        let input = outcome.borrowed()?;
        let user_login: UserLogin = match serde_json::from_str(input) {
            Ok(v) => v,
            Err(_) => {
                return Failure((
                    Status::UnprocessableEntity,
                    UserLoginError::Empty,
                ));
            },
        };

        if user_login.username == "" || user_login.password == "" {
            return Failure((
                Status::UnprocessableEntity,
                UserLoginError::Empty,
            ));
        }
        Success(UserLogin {
            username: user_login.username,
            password: user_login.password,
        })
    }
}
