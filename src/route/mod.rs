pub mod access_token;
pub mod authentication;
pub mod error;
pub mod message;
pub mod password_reset;
pub mod registration;
pub mod top;
pub mod user;

// TODO
pub const MAX_AGE: &str = "10800"; // 3 hours
pub const ORIGIN: &str = "http://127.0.0.1:3000";
pub const VARY: &str = "Accept-Encoding,Origin";
