/// The token for email verification.
///
/// In addition to general email confirmation, we use this verifacion token
/// also for an user activation by the user's primary email address.
use std::ops::Deref;

use redis::{Commands, RedisError};
use rocket::{Request, State, request};
use rocket::http::{Status, RawStr};
use rocket::request::FromRequest;
use rocket_slog::SyncLogger;

use config::Config;
use model::token::VerificationClaims;
use request::token::{AUTHORIZATION_HEADER_PREFIX, verify_token};
use ss::SsConn;

pub struct VerificationToken(pub String);

impl Deref for VerificationToken {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub enum VerificationTokenError {
    BadCount,
    Invalid,
    Missing,
}

// Extract and verify verification token given through HTTP Authorization
// header and a private cookie.
impl<'a, 'r> FromRequest<'a, 'r> for VerificationToken {
    type Error = VerificationTokenError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let headers: Vec<_> = req.headers().get("Authorization").collect();
        let logger = req.guard::<State<SyncLogger>>().unwrap();
        match headers.len() {
            1 => {
                let failure = request::Outcome::Failure((
                    Status::BadRequest,
                    VerificationTokenError::Invalid,
                ));
                let h = &headers[0];
                if !h.starts_with(AUTHORIZATION_HEADER_PREFIX) {
                    return failure;
                }

                // TODO:
                // * check X-Requested-With header
                // * check Origin and Referer header
                // * validate token format

                let token = h[AUTHORIZATION_HEADER_PREFIX.len()..].to_string();
                if !token.contains('.') {
                    return failure;
                }
                // NOTE:
                // append signature taken by session id to the parts extracted
                // from authorization header.
                let mut ss_conn = req.guard::<SsConn>().unwrap();
                let session_id: &'a RawStr = req
                    .get_query_value("s")
                    .and_then(|r| r.ok())
                    .unwrap_or_else(|| "".into());

                if session_id.is_empty() {
                    return failure;
                }

                let result: Result<String, RedisError> =
                    ss_conn.get(session_id.as_str()).map_err(|e| {
                        error!(logger, "error: {}", e);
                        e
                    });
                if result.is_err() {
                    return failure;
                }

                let verification_token = token + "." + &result.unwrap();
                let config = req.guard::<State<Config>>().unwrap();
                match verify_token::<VerificationClaims>(
                    &verification_token,
                    &config.verification_token_issuer,
                    &config.verification_token_secret,
                ) {
                    Ok(t) => request::Outcome::Success(VerificationToken(t)),
                    _ => failure,
                }
            },
            0 => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    VerificationTokenError::Missing,
                ))
            },
            _ => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    VerificationTokenError::BadCount,
                ))
            },
        }
    }
}
