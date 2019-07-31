use std::ops::Deref;

use rocket::{Request, State, request};
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
        let headers = req.headers();
        let auth: Vec<_> = headers.get("Authorization").collect();
        match auth.len() {
            0 => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthorizationTokenError::Missing,
                ))
            },
            1 => {
                // TODO: Improve format validation
                if !auth[0].starts_with("Bearer") {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthorizationTokenError::Invalid,
                    ));
                }

                let token = (&auth[0])[6..].to_string();
                if !token.contains('.') {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthorizationTokenError::Invalid,
                    ));
                }

                let config = req.guard::<State<Config>>().unwrap();
                match verify_authorization_token(&token, &config) {
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
