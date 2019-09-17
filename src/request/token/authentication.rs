/// The token for user authentication.
use std::ops::Deref;

use rocket::{Request, State, request};
use rocket::http::Status;
use rocket::request::FromRequest;

use config::Config;
use model::token::AuthenticationClaims;
use request::token::{AUTHORIZATION_HEADER_PREFIX, verify_token};

pub struct AuthenticationToken(pub String);

impl Deref for AuthenticationToken {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub enum AuthenticationTokenError {
    BadCount,
    Invalid,
    Missing,
}

// Extract and verify a token given through HTTP Authentication header.
//
// This should be handled within FromRequest for User.
impl<'a, 'r> FromRequest<'a, 'r> for AuthenticationToken {
    type Error = AuthenticationTokenError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let headers: Vec<_> = req.headers().get("Authorization").collect();
        match headers.len() {
            1 => {
                let h = &headers[0];
                if !h.starts_with(AUTHORIZATION_HEADER_PREFIX) {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthenticationTokenError::Invalid,
                    ));
                }

                // TODO:
                // * check X-Requested-With header
                // * check Origin and Referer header
                // * validate token format

                let mut token =
                    h[AUTHORIZATION_HEADER_PREFIX.len()..].to_string();
                if !token.contains('.') {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthenticationTokenError::Invalid,
                    ));
                }
                // NOTE:
                // append signature read from cookie to the parts sent as
                // a authentication header
                let cookies = req.cookies();
                token = cookies
                    .get("signature")
                    .map(|c| token + "." + c.value())
                    .or_else(|| Some("".to_string()))
                    .unwrap();

                let config = req.guard::<State<Config>>().unwrap();
                match verify_token::<AuthenticationClaims>(
                    &token,
                    &config.authentication_token_issuer,
                    &config.authentication_token_secret,
                ) {
                    Ok(t) => request::Outcome::Success(AuthenticationToken(t)),
                    _ => {
                        request::Outcome::Failure((
                            Status::BadRequest,
                            AuthenticationTokenError::Invalid,
                        ))
                    },
                }
            },
            0 => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthenticationTokenError::Missing,
                ))
            },
            _ => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthenticationTokenError::BadCount,
                ))
            },
        }
    }
}
