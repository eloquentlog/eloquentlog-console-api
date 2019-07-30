use std::io::{self, Read};

use rocket::{Data, Outcome::*, Request, State, request};
use rocket::data::{self, FromData, Transform, Transformed};
use rocket::http::Status;
use rocket::request::FromRequest;
use rocket_slog::SyncLogger;
use serde_json;

use config::Config;
use db::DbConn;
use model::token::AuthorizationClaims;
use model::user::User;
use request::token::AuthorizationToken;

/// User
impl<'a, 'r> FromRequest<'a, 'r> for &'a User {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> request::Outcome<&'a User, ()> {
        let signin = req.local_cache(|| {
            let config = req.guard::<State<Config>>().unwrap();
            let db_conn = req.guard::<DbConn>().unwrap();
            let logger = req.guard::<SyncLogger>().unwrap();

            let token =
                req.local_cache(|| req.guard::<AuthorizationToken>().unwrap());

            User::find_by_token::<AuthorizationClaims>(
                &token,
                &config.authorization_token_issuer,
                &config.authorization_token_secret,
                &db_conn,
                &logger,
            )
        });
        if let Some(ref user) = signin {
            return request::Outcome::Success(user);
        }
        request::Outcome::Forward(())
    }
}

/// UserSignUp
#[derive(Clone, Deserialize)]
pub struct UserSignUp {
    pub email: String,
    pub name: Option<String>,
    pub username: String,
    pub password: String,
}

impl Default for UserSignUp {
    fn default() -> Self {
        Self {
            email: "".to_string(),
            name: None,
            username: "".to_string(),
            password: "".to_string(),
        }
    }
}

/// UserSignin
pub enum UserSigninError {
    Io(io::Error),
    Empty,
}

const USER_SIGNIN_LENGTH_LIMIT: u64 = 256;

#[derive(Clone, Debug, Deserialize)]
pub struct UserSignin {
    pub username: String,
    pub password: String,
}

impl<'v> FromData<'v> for UserSignin {
    type Error = UserSigninError;
    type Owned = String;
    type Borrowed = str;

    fn transform(
        _: &Request,
        data: Data,
    ) -> Transform<data::Outcome<Self::Owned, Self::Error>>
    {
        let mut stream = data.open().take(USER_SIGNIN_LENGTH_LIMIT);
        let mut string =
            String::with_capacity((USER_SIGNIN_LENGTH_LIMIT / 2) as usize);
        let outcome = match stream.read_to_string(&mut string) {
            Ok(_) => Success(string),
            Err(e) => {
                Failure((Status::InternalServerError, UserSigninError::Io(e)))
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
        let user_signin: UserSignin = match serde_json::from_str(input) {
            Ok(v) => v,
            Err(_) => {
                return Failure((
                    Status::UnprocessableEntity,
                    UserSigninError::Empty,
                ));
            },
        };

        if user_signin.username == "" || user_signin.password == "" {
            return Failure((
                Status::UnprocessableEntity,
                UserSigninError::Empty,
            ));
        }
        Success(UserSignin {
            username: user_signin.username,
            password: user_signin.password,
        })
    }
}
