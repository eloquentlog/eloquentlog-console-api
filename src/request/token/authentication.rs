/// The token for user authentication.
use std::ops::Deref;

use rocket::{Request, State};
use rocket::request::{FromRequest, Outcome};
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::model::token::AuthenticationClaims;
use crate::request::token::{AUTHORIZATION_HEADER_PREFIX, verify_token};

use crate::{bad_request_by, unprocessable_entity_by};

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
    Unknown,
}

// Extract and verify a token given through HTTP Authentication header.
//
// This should be handled within FromRequest for User.
impl<'a, 'r> FromRequest<'a, 'r> for AuthenticationToken {
    type Error = AuthenticationTokenError;

    fn from_request(req: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let logger = req.guard::<State<SyncLogger>>().unwrap();

        if req.headers().get_one("X-Requested-With") != Some("XMLHttpRequest") {
            error!(logger, "request: {}", req);
            return bad_request_by!(AuthenticationTokenError::Invalid);
        }

        let headers: Vec<_> = req.headers().get("Authorization").collect();
        match headers.len() {
            1 => {
                let h = &headers[0];
                if !h.starts_with(AUTHORIZATION_HEADER_PREFIX) {
                    return bad_request_by!(AuthenticationTokenError::Invalid);
                }

                // TODO:
                // * check Origin and Referer header
                // * validate token format

                let token = h[AUTHORIZATION_HEADER_PREFIX.len()..].to_string();
                if !token.contains('.') {
                    return bad_request_by!(AuthenticationTokenError::Invalid);
                }

                // NOTE:
                // append signature taken by using session id to the parts
                // extracted from authorization header.
                // TOD: use get_private
                let authentication_token: String = req
                    .cookies()
                    .get("sign")
                    .map(|c| token + "." + c.value())
                    .or_else(|| Some("".to_string()))
                    .unwrap();

                if authentication_token.is_empty() {
                    return bad_request_by!(AuthenticationTokenError::Invalid);
                }

                let config = req.guard::<State<Config>>().unwrap();
                match verify_token::<AuthenticationClaims>(
                    &authentication_token,
                    &config.authentication_token_issuer,
                    &config.authentication_token_secret,
                ) {
                    Ok(t) => Outcome::Success(AuthenticationToken(t)),
                    Err(e) => {
                        error!(logger, "error: {}", e);
                        unprocessable_entity_by!(
                            AuthenticationTokenError::Invalid
                        )
                    },
                }
            },
            0 => bad_request_by!(AuthenticationTokenError::Missing),
            _ => bad_request_by!(AuthenticationTokenError::BadCount),
        }
    }
}
