/// The token for user authentication.
use std::ops::Deref;

use rocket::{Request, State};
use rocket::request::{FromRequest, Outcome};
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::model::token::AuthenticationClaims;
use crate::request::token::{
    AUTHORIZATION_HEADER_PREFIX, AUTHORIZATION_HEADER_TOKEN_PREFIX, TokenType,
    verify_token,
};

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

        let token_type = req
            .guard::<TokenType>()
            .failure_then(|v| Outcome::Failure((v.0, Self::Error::Invalid)))?;

        if req.headers().get_one("X-Requested-With") != Some("XMLHttpRequest") {
            error!(logger, "request: {}", req);
            return bad_request_by!(AuthenticationTokenError::Invalid);
        }

        let headers: Vec<_> = req.headers().get("Authorization").collect();
        match headers.len() {
            1 => {
                let h = &headers[0];
                let mut token = match token_type {
                    TokenType::BrowserCookieToken => {
                        let length = AUTHORIZATION_HEADER_PREFIX.len();
                        h[length..].to_string()
                    },
                    TokenType::PersonalAccessToken => {
                        let length = AUTHORIZATION_HEADER_TOKEN_PREFIX.len();
                        h[length..].to_string()
                    },
                };

                // TODO:
                // * check Origin and Referer header
                // * validate token format

                if token.is_empty() || !token.contains('.') {
                    return bad_request_by!(AuthenticationTokenError::Invalid);
                }

                // NOTE:
                // append signature taken by using session id to the parts
                // extracted from authorization header.
                // TODO: use get_private
                if token_type == TokenType::BrowserCookieToken {
                    token = req
                        .cookies()
                        .get("sign")
                        .map(|c| token + "." + c.value())
                        .or_else(|| Some("".to_string()))
                        .unwrap();
                }

                if token.is_empty() {
                    return bad_request_by!(AuthenticationTokenError::Invalid);
                }

                let config = req.guard::<State<Config>>().unwrap();
                match verify_token::<AuthenticationClaims>(
                    &token,
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
