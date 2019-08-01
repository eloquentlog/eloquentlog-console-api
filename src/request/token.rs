use std::ops::Deref;

use rocket::{Request, State, request};
use rocket::http::Status;
use rocket::request::FromRequest;

use config::Config;
use model::token::{AuthorizationClaims, Claims};

const AUTHORIZATION_HEADER_PREFIX: &str = "Bearer ";

pub struct AuthorizationToken(pub String);

impl Deref for AuthorizationToken {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

fn verify_authorization_token(
    value: &str,
    config: &Config,
) -> Result<String, String>
{
    // as validations
    let _ = AuthorizationClaims::decode(
        &value,
        &config.authorization_token_issuer,
        &config.authorization_token_secret,
    )
    .expect("Invalid value");
    Ok(value.to_string())
}

#[derive(Debug)]
pub enum AuthorizationTokenError {
    BadCount,
    Invalid,
    Missing,
}

// Extract and verify a token given through HTTP Authorization header.
//
// This should be handled within FromRequest for User.
impl<'a, 'r> FromRequest<'a, 'r> for AuthorizationToken {
    type Error = AuthorizationTokenError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let headers: Vec<_> = req.headers().get("Authorization").collect();
        match headers.len() {
            0 => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthorizationTokenError::Missing,
                ))
            },
            1 => {
                let h = &headers[0];
                if !h.starts_with(AUTHORIZATION_HEADER_PREFIX) {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthorizationTokenError::Invalid,
                    ));
                }

                // TODO: validate format
                let token = h[AUTHORIZATION_HEADER_PREFIX.len()..].to_string();
                if !token.contains('.') {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthorizationTokenError::Invalid,
                    ));
                }

                let config = req.guard::<State<Config>>().unwrap();
                match verify_authorization_token(&token, &config) {
                    Ok(t) => request::Outcome::Success(AuthorizationToken(t)),
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
