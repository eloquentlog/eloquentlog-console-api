use std::io::{self, Read};

use rocket::{Data, Outcome::*, Request, State, request};
use rocket::data::{self, FromData, Transform, Transformed};
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket_slog::SyncLogger;
use serde_json;

use config::Config;
use db::DbConn;
use model::ticket::AuthorizationClaims;
use model::user::User;
use request::ticket::AuthorizationTicket;

/// User
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> request::Outcome<User, ()> {
        let ticket = req.guard::<AuthorizationTicket>().unwrap();

        let config = req.guard::<State<Config>>()?;
        let conn = req.guard::<DbConn>()?;
        let logger = req.guard::<SyncLogger>()?;

        if let Some(user) = User::find_by_ticket::<AuthorizationClaims>(
            &ticket,
            &config.authorization_ticket_issuer,
            &config.authorization_ticket_secret,
            &conn,
            &logger,
        ) {
            return request::Outcome::Success(user);
        }
        request::Outcome::Forward(())
    }
}

/// UserSignUp
#[derive(Clone, Deserialize)]
pub struct UserSignUp {
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub password: String,
}

impl Default for UserSignUp {
    fn default() -> Self {
        Self {
            name: None,
            username: None,
            email: "".to_string(),
            password: "".to_string(),
        }
    }
}

/// UserSignIn
pub enum UserSignInError {
    Io(io::Error),
    Empty,
}

const USER_SIGN_IN_LENGTH_LIMIT: u64 = 256;

#[derive(Clone, Debug, Deserialize)]
pub struct UserSignIn {
    pub username: String,
    pub password: String,
}

impl<'v> FromData<'v> for UserSignIn {
    type Error = UserSignInError;
    type Owned = String;
    type Borrowed = str;

    fn transform(
        _: &Request,
        data: Data,
    ) -> Transform<data::Outcome<Self::Owned, Self::Error>>
    {
        let mut stream = data.open().take(USER_SIGN_IN_LENGTH_LIMIT);
        let mut string =
            String::with_capacity((USER_SIGN_IN_LENGTH_LIMIT / 2) as usize);
        let outcome = match stream.read_to_string(&mut string) {
            Ok(_) => Success(string),
            Err(e) => {
                Failure((Status::InternalServerError, UserSignInError::Io(e)))
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
        let user_login: UserSignIn = match serde_json::from_str(input) {
            Ok(v) => v,
            Err(_) => {
                return Failure((
                    Status::UnprocessableEntity,
                    UserSignInError::Empty,
                ));
            },
        };

        if user_login.username == "" || user_login.password == "" {
            return Failure((
                Status::UnprocessableEntity,
                UserSignInError::Empty,
            ));
        }
        Success(UserSignIn {
            username: user_login.username,
            password: user_login.password,
        })
    }
}
