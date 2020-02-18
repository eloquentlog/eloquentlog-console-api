use rocket::request::FromParam;
use rocket::http::RawStr;

use crate::model::access_token::AgentType;

impl<'r> FromParam<'r> for AgentType {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        Ok(AgentType::from(param.as_str().to_string()))
    }
}
