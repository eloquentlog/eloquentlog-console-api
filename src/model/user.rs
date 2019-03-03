use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: Vec<u8>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<User {username}>", username = self.username)
    }
}

impl User {
    pub fn verify_password(&self, _password: &str) -> bool {
        // TODO
        true
    }
}
