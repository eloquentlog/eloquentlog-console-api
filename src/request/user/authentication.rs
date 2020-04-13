use std::io::{self, Read};

use rocket::{Data, Outcome::*, Request};
use rocket::data::{self, FromData, Transform, Transformed};
use rocket::http::Status;

/// UserAuthentication
pub enum UserAuthenticationError {
    Io(io::Error),
    Empty,
}

const USER_AUTHENTICATION_LENGTH_LIMIT: u64 = 256;

#[derive(Clone, Debug, Deserialize)]
pub struct UserAuthentication {
    pub username: String,
    pub password: String,
}

impl<'v> FromData<'v> for UserAuthentication {
    type Error = UserAuthenticationError;
    type Owned = String;
    type Borrowed = str;

    fn transform(
        _: &Request,
        data: Data,
    ) -> Transform<data::Outcome<Self::Owned, Self::Error>>
    {
        let mut stream = data.open().take(USER_AUTHENTICATION_LENGTH_LIMIT);
        let mut string = String::with_capacity(
            (USER_AUTHENTICATION_LENGTH_LIMIT / 2) as usize,
        );
        let outcome = match stream.read_to_string(&mut string) {
            Ok(_) => Success(string),
            Err(e) => {
                Failure((
                    Status::InternalServerError,
                    UserAuthenticationError::Io(e),
                ))
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
        let authentication: UserAuthentication =
            match serde_json::from_str(input) {
                Ok(v) => v,
                Err(_) => {
                    return Failure((
                        Status::UnprocessableEntity,
                        UserAuthenticationError::Empty,
                    ));
                },
            };

        if authentication.username == "" || authentication.password == "" {
            return Failure((
                Status::UnprocessableEntity,
                UserAuthenticationError::Empty,
            ));
        }
        Success(authentication)
    }
}
