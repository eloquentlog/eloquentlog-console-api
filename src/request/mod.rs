pub mod agent_type;
pub mod message;
pub mod password_reset;
pub mod token;
pub mod user;

type ID = usize;

#[macro_export]
macro_rules! bad_request_by {
    ($reason:expr) => {
        ::rocket::request::Outcome::Failure((
            ::rocket::http::Status::BadRequest,
            $reason,
        ))
    };
}

#[macro_export]
macro_rules! not_found_by {
    ($reason:expr) => {
        ::rocket::request::Outcome::Failure((
            ::rocket::http::Status::NotFound,
            $reason,
        ))
    };
}

#[macro_export]
macro_rules! unprocessable_entity_by {
    ($reason:expr) => {
        ::rocket::request::Outcome::Failure((
            ::rocket::http::Status::UnprocessableEntity,
            $reason,
        ))
    };
}
