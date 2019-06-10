use std::ops::Deref;

use rocket::{Request, State, request};
use rocket::http::Status;
use rocket::request::FromRequest;

use config::Config;
use model::voucher::{AuthorizationClaims, Claims};
use route::AUTHORIZATION_HEADER_KEY;

pub struct AuthorizationVoucher(pub String);

impl Deref for AuthorizationVoucher {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

fn verify_authorization_voucher(
    value: &str,
    config: &Config,
) -> Result<String, String>
{
    // as validations
    let _ = AuthorizationClaims::decode(
        &value,
        &config.authorization_voucher_issuer,
        &config.authorization_voucher_secret,
    )
    .expect("Invalid value");
    Ok(value.to_string())
}

#[derive(Debug)]
pub enum AuthorizationVoucherError {
    BadCount,
    Invalid,
    Missing,
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthorizationVoucher {
    type Error = AuthorizationVoucherError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let headers = req.headers();
        let keys: Vec<_> = headers.get(AUTHORIZATION_HEADER_KEY).collect();
        match keys.len() {
            0 => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthorizationVoucherError::Missing,
                ))
            },
            1 => {
                let value = keys[0];
                if !value.contains('.') {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthorizationVoucherError::Invalid,
                    ));
                }

                let config = req.guard::<State<Config>>().unwrap();
                match verify_authorization_voucher(value, &config) {
                    Ok(v) => request::Outcome::Success(AuthorizationVoucher(v)),
                    _ => {
                        request::Outcome::Failure((
                            Status::BadRequest,
                            AuthorizationVoucherError::Invalid,
                        ))
                    },
                }
            },
            _ => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthorizationVoucherError::BadCount,
                ))
            },
        }
    }
}
