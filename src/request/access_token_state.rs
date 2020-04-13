use rocket::request::FromParam;
use rocket::http::RawStr;

use crate::model::access_token::AccessTokenState;

impl<'r> FromParam<'r> for AccessTokenState {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        Ok(AccessTokenState::from(param.as_str().to_string()))
    }
}
