pub mod authentication;
pub mod error;
pub mod message;
pub mod password_reset;
pub mod registration;
pub mod user;
pub mod top;

// TODO
pub const ORIGIN: &str = "http://127.0.0.1:3000";
pub const MAX_AGE: &str = "10800"; // 3 hours
pub const VARY: &str = "Accept-Encoding,Origin";

pub const AUTHORIZATION_HEADER_KEY: &str = "X-Eloquentlog-Authorization-Token";
