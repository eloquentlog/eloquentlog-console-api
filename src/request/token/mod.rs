pub mod authentication;
pub mod verification;

use jsonwebtoken::errors::Error;

use crate::model::token::Claims;

const AUTHORIZATION_HEADER_PREFIX: &str = "Bearer ";

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
