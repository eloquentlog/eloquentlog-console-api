use std::ops::Deref;

use rocket::{Request, State};
use rocket::request;
use rocket::http::Status;
use rocket::request::FromRequest;

use config::Config;
use model::user::Claims;

pub struct AuthToken(pub String); // JWT

impl Deref for AuthToken {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

fn decode(value: &str, config: &Config) -> Result<String, String> {
    // with validation
    let _ = Claims::decode(&value, &config.jwt_issuer, &config.jwt_secret)
        .expect("Invalid token")
        .claims;
    Ok(value.to_string())
}

#[derive(Debug)]
pub enum AuthTokenError {
    BadCount,
    Invalid,
    Missing,
}

pub const X_ELOQUENTLOG_AUTH_KEY: &str = "X-Eloquentlog-Auth-Token";

impl<'a, 'r> FromRequest<'a, 'r> for AuthToken {
    type Error = AuthTokenError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let headers = req.headers();
        let keys: Vec<_> = headers.get(X_ELOQUENTLOG_AUTH_KEY).collect();
        match keys.len() {
            0 => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthTokenError::Missing,
                ))
            },
            1 => {
                let value = keys[0];
                if !value.contains('.') {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthTokenError::Invalid,
                    ));
                }

                let config = req.guard::<State<Config>>().unwrap();
                match decode(value, &config) {
                    Ok(v) => request::Outcome::Success(AuthToken(v)),
                    _ => {
                        request::Outcome::Failure((
                            Status::BadRequest,
                            AuthTokenError::Invalid,
                        ))
                    },
                }
            },
            _ => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthTokenError::BadCount,
                ))
            },
        }
    }
}
