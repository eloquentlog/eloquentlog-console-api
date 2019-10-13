/// The token for email verification.
///
/// In addition to general email confirmation, we use this verifacion token
/// also for an user activation by the user's primary email address.
use std::ops::Deref;

use redis::{Commands, RedisError};
use rocket::{Request, State, request, request::Outcome};
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
    Expired,
    Invalid,
    Missing,
    Unknown,
}

fn respond_as_expired() -> Outcome<VerificationToken, VerificationTokenError> {
    Outcome::Failure((
        Status::UnprocessableEntity,
        VerificationTokenError::Expired,
    ))
}

fn respond_as_invalid() -> Outcome<VerificationToken, VerificationTokenError> {
    Outcome::Failure((Status::BadRequest, VerificationTokenError::Invalid))
}

fn respond_as_missing() -> Outcome<VerificationToken, VerificationTokenError> {
    Outcome::Failure((Status::BadRequest, VerificationTokenError::Missing))
}

fn respond_as_unknown() -> Outcome<VerificationToken, VerificationTokenError> {
    Outcome::Failure((Status::NotFound, VerificationTokenError::Unknown))
}

// Extract and verify verification token given through HTTP Authorization
// header and a private cookie.
impl<'a, 'r> FromRequest<'a, 'r> for VerificationToken {
    type Error = VerificationTokenError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let logger = req.guard::<State<SyncLogger>>().unwrap();

        if req.headers().get_one("X-Requested-With") != Some("XMLHttpRequest") {
            error!(logger, "request: {}", req);
            return respond_as_invalid();
        }

        let headers: Vec<_> = req.headers().get("Authorization").collect();
        match headers.len() {
            1 => {
                let h = &headers[0];
                if !h.starts_with(AUTHORIZATION_HEADER_PREFIX) {
                    return respond_as_invalid();
                }

                // TODO:
                // * check Origin and Referer header
                // * validate token format

                let token = h[AUTHORIZATION_HEADER_PREFIX.len()..].to_string();
                if !token.contains('.') {
                    return respond_as_invalid();
                }
                // NOTE:
                // append signature taken by session id to the parts extracted
                // from authorization header.
                let mut ss_conn = req.guard::<SsConn>().unwrap();
                // /_api/password/reset/<...> and /_api/user/activate/<...>
                let session_id: &'a RawStr = req
                    .get_param(2)
                    .and_then(|r| r.ok())
                    .unwrap_or_else(|| "".into());

                if session_id.is_empty() {
                    return respond_as_invalid();
                }

                let result: Result<String, RedisError> =
                    ss_conn.get(session_id.as_str()).map_err(|e| {
                        error!(logger, "error: {}", e);
                        e
                    });
                if result.is_err() {
                    return respond_as_unknown();
                }

                let verification_token = token + "." + &result.unwrap();
                let config = req.guard::<State<Config>>().unwrap();
                match verify_token::<VerificationClaims>(
                    &verification_token,
                    &config.verification_token_issuer,
                    &config.verification_token_secret,
                ) {
                    Ok(t) => request::Outcome::Success(VerificationToken(t)),
                    Err(e) => {
                        error!(logger, "error: {}", e);
                        respond_as_expired()
                    },
                }
            },
            0 => {
                error!(logger, "error: Authorization header is missing");
                respond_as_missing()
            },
            _ => respond_as_invalid(),
        }
    }
}
