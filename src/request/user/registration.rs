/// UserRegistration (signup)
#[derive(Clone, Deserialize)]
pub struct UserRegistration {
    pub email: String,
    pub name: Option<String>,
    pub username: String,
    pub password: String,
}

impl Default for UserRegistration {
    fn default() -> Self {
        Self {
            email: "".to_string(),
            name: None,
            username: "".to_string(),
            password: "".to_string(),
        }
    }
}
