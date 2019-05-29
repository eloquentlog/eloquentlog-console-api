use std::ops::Deref;

use rocket::{Request, State};
use rocket::request;
use rocket::http::Status;
use rocket::request::FromRequest;

use config::Config;
use model::token::{AuthorizationClaims, Claims};

pub struct AuthorizationToken(pub String);

impl Deref for AuthorizationToken {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

fn decode_authorization_token(
    value: &str,
    config: &Config,
) -> Result<String, String>
{
    // with validation
    let _ = AuthorizationClaims::decode(
        &value,
        &config.authorization_token_issuer,
        &config.authorization_token_secret,
    )
    .expect("Invalid token");
    Ok(value.to_string())
}

#[derive(Debug)]
pub enum AuthorizationTokenError {
    BadCount,
    Invalid,
    Missing,
}

pub const AUTHORIZATION_HEADER_KEY: &str = "X-Eloquentlog-Authorization-Token";

impl<'a, 'r> FromRequest<'a, 'r> for AuthorizationToken {
    type Error = AuthorizationTokenError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let headers = req.headers();
        let keys: Vec<_> = headers.get(AUTHORIZATION_HEADER_KEY).collect();
        match keys.len() {
            0 => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthorizationTokenError::Missing,
                ))
            },
            1 => {
                let value = keys[0];
                if !value.contains('.') {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthorizationTokenError::Invalid,
                    ));
                }

                let config = req.guard::<State<Config>>().unwrap();
                match decode_authorization_token(value, &config) {
                    Ok(v) => request::Outcome::Success(AuthorizationToken(v)),
                    _ => {
                        request::Outcome::Failure((
                            Status::BadRequest,
                            AuthorizationTokenError::Invalid,
                        ))
                    },
                }
            },
            _ => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthorizationTokenError::BadCount,
                ))
            },
        }
    }
}
