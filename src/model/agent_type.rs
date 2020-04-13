//! A type represents client or personal access token for
//! access_tokens table.
use std::fmt;
use std::io::Write;
use std::slice::Iter;

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::Serialize;

static AGENT_TYPES: [AgentType; 2] = [AgentType::Client, AgentType::Person];

#[derive(QueryId, SqlType, Clone)]
#[postgres(type_name = "e_agent_type")]
pub struct EAgentType;

#[derive(AsExpression, Clone, Debug, FromSqlRow, PartialEq, Serialize)]
#[sql_type = "EAgentType"]
pub enum AgentType {
    Client, // default
    Person,
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AgentType::Client => write!(f, "client"),
            AgentType::Person => write!(f, "person"),
        }
    }
}

impl ToSql<EAgentType, Pg> for AgentType {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            AgentType::Client => out.write_all(b"client")?,
            AgentType::Person => out.write_all(b"person")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<EAgentType, Pg> for AgentType {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"client" => Ok(AgentType::Client),
            b"person" => Ok(AgentType::Person),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<String> for AgentType {
    fn from(s: String) -> Self {
        match s.to_ascii_uppercase().as_ref() {
            "CLIENT" => AgentType::Client,
            "PERSON" => AgentType::Person,
            _ => AgentType::Client,
        }
    }
}

impl AgentType {
    pub fn iter() -> Iter<'static, AgentType> {
        AGENT_TYPES.iter()
    }

    pub fn as_vec() -> Vec<AgentType> {
        AgentType::iter().cloned().collect()
    }

    pub fn is_person(&self) -> bool {
        self == &AgentType::Person
    }

    pub fn is_client(&self) -> bool {
        self == &AgentType::Client
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(AgentType::Client, AgentType::from("client".to_string()));
        assert_eq!(AgentType::Client, AgentType::from("Client".to_string()));
        assert_eq!(AgentType::Client, AgentType::from("CLIENT".to_string()));

        assert_eq!(AgentType::Person, AgentType::from("person".to_string()));
        assert_eq!(AgentType::Person, AgentType::from("Person".to_string()));
        assert_eq!(AgentType::Person, AgentType::from("PERSON".to_string()));

        // default
        assert_eq!(AgentType::Client, AgentType::from("unknown".to_string()));
    }

    #[test]
    fn test_fmt() {
        assert_eq!("client", format!("{}", AgentType::Client));
        assert_eq!("person", format!("{}", AgentType::Person));
    }

    #[test]
    fn test_as_vec() {
        assert_eq!(
            vec![AgentType::Client, AgentType::Person],
            AgentType::as_vec()
        );
    }

    #[test]
    fn test_is_person() {
        assert!(AgentType::Person.is_person());
        assert!(!AgentType::Client.is_person());
    }

    #[test]
    fn test_is_client() {
        assert!(!AgentType::Person.is_client());
        assert!(AgentType::Client.is_client());
    }
}
