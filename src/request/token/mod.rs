pub mod authentication;
pub mod verification;

use jsonwebtoken::errors::Error;
use rocket::{Request, request};
use rocket::request::{FromRequest, Outcome};

use crate::unprocessable_entity_by;
use crate::model::token::Claims;

const AUTHORIZATION_HEADER_PREFIX: &str = "Bearer ";
const AUTHORIZATION_HEADER_TOKEN_PREFIX: &str = "Token ";

// NOTE: this function does not check value in database.
fn verify_token<T>(
    value: &str,
    issuer: &str,
    secret: &str,
) -> Result<String, Error>
where
    T: Claims,
{
    let _ = T::decode(value, issuer, secret)?;
    Ok(value.to_string())
}

/// TokenType
#[derive(PartialEq)]
pub enum TokenType {
    BrowserCookieToken,
    PersonalAccessToken,
}

#[derive(Debug)]
pub enum TokenTypeError {
    Unknown,
}

impl<'a, 'r> FromRequest<'a, 'r> for TokenType {
    type Error = TokenTypeError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let headers: Vec<_> = req.headers().get("Authorization").collect();
        if headers.len() == 1 {
            let header = headers.first().unwrap();
            if header.starts_with(AUTHORIZATION_HEADER_PREFIX) {
                return Outcome::Success(Self::BrowserCookieToken);
            } else if header.starts_with(AUTHORIZATION_HEADER_TOKEN_PREFIX) {
                return Outcome::Success(Self::PersonalAccessToken);
            }
        }
        unprocessable_entity_by!(Self::Error::Unknown)
    }
}
