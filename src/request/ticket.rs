use std::ops::Deref;

use rocket::{Request, State, request};
use rocket::http::Status;
use rocket::request::FromRequest;

use config::Config;
use model::ticket::{AuthorizationClaims, Claims};
use route::AUTHORIZATION_HEADER_KEY;

pub struct AuthorizationTicket(pub String);

impl Deref for AuthorizationTicket {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

fn verify_authorization_ticket(
    value: &str,
    config: &Config,
) -> Result<String, String>
{
    // as validations
    let _ = AuthorizationClaims::decode(
        &value,
        &config.authorization_ticket_issuer,
        &config.authorization_ticket_secret,
    )
    .expect("Invalid value");
    Ok(value.to_string())
}

#[derive(Debug)]
pub enum AuthorizationTicketError {
    BadCount,
    Invalid,
    Missing,
}

// Extract and verify a token given through HTTP Authorization header.
//
// This should be handled within FromRequest for User.
impl<'a, 'r> FromRequest<'a, 'r> for AuthorizationTicket {
    type Error = AuthorizationTicketError;

    fn from_request(
        req: &'a Request<'r>,
    ) -> request::Outcome<Self, Self::Error> {
        let headers = req.headers();
        let keys: Vec<_> = headers.get(AUTHORIZATION_HEADER_KEY).collect();
        match keys.len() {
            0 => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthorizationTicketError::Missing,
                ))
            },
            1 => {
                let value = keys[0];
                if !value.contains('.') {
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        AuthorizationTicketError::Invalid,
                    ));
                }

                let config = req.guard::<State<Config>>().unwrap();
                match verify_authorization_ticket(value, &config) {
                    Ok(v) => request::Outcome::Success(AuthorizationTicket(v)),
                    _ => {
                        request::Outcome::Failure((
                            Status::BadRequest,
                            AuthorizationTicketError::Invalid,
                        ))
                    },
                }
            },
            _ => {
                request::Outcome::Failure((
                    Status::BadRequest,
                    AuthorizationTicketError::BadCount,
                ))
            },
        }
    }
}
