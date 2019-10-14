/// PasswordResetRequest
#[derive(Clone, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

impl Default for PasswordResetRequest {
    fn default() -> Self {
        Self {
            email: "".to_string(),
        }
    }
}

/// PasswordResetUpdate
#[derive(Clone, Deserialize)]
pub struct PasswordResetUpdate {
    pub new_password: String,
}

impl Default for PasswordResetUpdate {
    fn default() -> Self {
        Self {
            new_password: "".to_string(),
        }
    }
}

/// PasswordReset
#[derive(Clone, Deserialize)]
pub struct PasswordReset {
    pub password: String,
    pub username: String,
}

impl Default for PasswordReset {
    fn default() -> Self {
        Self {
            password: "".to_string(),
            username: "".to_string(),
        }
    }
}
