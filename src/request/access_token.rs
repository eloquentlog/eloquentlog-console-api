use std::io::{self, Read};

use rocket::{Data, Outcome::*, Request, State};
use rocket::data::{self, FromData, Transform, Transformed};
use rocket::http::Status;
use rocket_slog::SyncLogger;
use serde::{Deserialize, Deserializer};

use crate::model::access_token::AccessTokenState;

/// AccessTokenError
pub enum AccessTokenError {
    Io(io::Error),
    Empty,
}

const ACCESS_TOKEN_LENGTH_LIMIT: u64 = 128;

fn parse_state<'de, D>(d: D) -> Result<AccessTokenState, D::Error>
where D: Deserializer<'de> {
    Deserialize::deserialize(d).map(|x: Option<_>| {
        AccessTokenState::from(x.unwrap_or_else(|| "disabled".to_owned()))
    })
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccessTokenData {
    pub access_token: AccessTokenObject,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccessTokenObject {
    #[serde(deserialize_with = "parse_state")]
    pub state: AccessTokenState,
}

impl<'v> FromData<'v> for AccessTokenData {
    type Error = AccessTokenError;
    type Owned = String;
    type Borrowed = str;

    fn transform(
        req: &Request,
        data: Data,
    ) -> Transform<data::Outcome<Self::Owned, Self::Error>> {
        let logger = req.guard::<State<SyncLogger>>().unwrap();

        let mut stream = data.open().take(ACCESS_TOKEN_LENGTH_LIMIT);
        let mut string =
            String::with_capacity((ACCESS_TOKEN_LENGTH_LIMIT / 2) as usize);
        let out = match stream.read_to_string(&mut string) {
            Ok(_) => Success(string),
            Err(e) => {
                error!(logger, "err: {}", e);
                Failure((Status::InternalServerError, AccessTokenError::Io(e)))
            },
        };

        Transform::Borrowed(out)
    }

    fn from_data(
        req: &Request,
        outcome: Transformed<'v, Self>,
    ) -> data::Outcome<Self, Self::Error> {
        let logger = req.guard::<State<SyncLogger>>().unwrap();

        let input = outcome.borrowed()?;
        let out: Self = match serde_json::from_str(input) {
            Ok(v) => v,
            Err(e) => {
                error!(logger, "err: {}", e);
                return Failure((
                    Status::UnprocessableEntity,
                    AccessTokenError::Empty,
                ));
            },
        };

        Success(out)
    }
}
