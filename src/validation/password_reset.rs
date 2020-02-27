use std::result::Result;

use accord::validators::length;
use diesel::PgConnection;
use rocket_contrib::json::Json;

use crate::logger::Logger;
use crate::request::password_reset::PasswordReset as RequestData;
use crate::validation::*;

pub struct Validator<'a> {
    // conn: &'a PgConnection,
    data: &'a Json<RequestData>,
    logger: &'a Logger,
}

impl<'a> Validator<'a> {
    pub fn new(
        _: &'a PgConnection,
        data: &'a Json<RequestData>,
        logger: &'a Logger,
    ) -> Self
    {
        Self { data, logger }
    }

    #[allow(clippy::redundant_closure)]
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let result = rules! {
            // TODO: share this rule with a validation for user registration
            "password" => self.data.0.password => [
                contain_any(CHARS_LOWER, "a-z"),
                contain_any(CHARS_UPPER, "A-Z"),
                contain_any(DIGITS, "0-9"),
                not_overlap_with("username")(self.data.0.username.to_string()),
                length(8, 1024)
            ]
        };

        let mut errors: Vec<ValidationError> = vec![];

        if let Err(v) = result {
            // MultipleError to Vec<ValidationError>
            errors =
                v.0.iter()
                    .map(|e| {
                        ValidationError {
                            field: e.tag.to_string(),
                            messages: e
                                .invalids
                                .iter()
                                .map(|i| i.human_readable.to_string())
                                .collect(),
                        }
                    })
                    .collect();
        }

        if !errors.is_empty() {
            for e in &errors {
                info!(
                    self.logger,
                    "validation error: {} {}",
                    e.field,
                    e.messages.join(",")
                );
            }
            return Err(errors);
        }
        Ok(())
    }
}

#[rustfmt::skip::attributes(rstest)]
#[cfg(test)]
mod test {
    use super::*;

    use rocket_contrib::json::Json;
    use rstest::rstest;

    use crate::model::test::run;

    #[test]
    fn test_validate_password_is_too_short() {
        run(|_, _, logger| {
            let data = &Json(RequestData {
                username: "username".to_string(),
                password: "Sh0rt".to_string(),
            });
            let v = Validator { data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("password", errors[0].field);
                assert_eq!(
                    vec!["Must contain more than 8 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_password_is_too_long() {
        run(|_, _, logger| {
            let data = &Json(RequestData {
                username: "username".to_string(),
                password: "L0ng".repeat(257),
            });
            let v = Validator { data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("password", errors[0].field);
                assert_eq!(
                    vec!["Must contain less than 1024 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_password_equals_username() {
        run(|_, _, logger| {
            let data = &Json(RequestData {
                username: "Passw0rd".to_string(),
                password: "Passw0rd".to_string(),
            });
            let v = Validator { data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("password", errors[0].field);
                assert_eq!(
                    vec!["Must not overlap with username"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[rstest(
        username, password,
        case("u$ern4mE", "Myu$ern4mE1sPart0fPassw0rd"),
        case("u$ern4mE", "u$ern4mE1sMyPassw0rd"),
        case("u$ern4mE", "u$ern4mE"),
        case("u$ern4mE", "Myu$ern4mE"),
        ::trace
    )]
    #[test]
    fn test_validate_password_contains_username(
        username: &'static str,
        password: &'static str,
    )
    {
        run(|_, _, logger| {
            let data = &Json(RequestData {
                username: username.to_string(),
                password: password.to_string(),
            });
            let v = Validator { data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("password", errors[0].field);
                assert_eq!(
                    vec!["Must not overlap with username"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[rstest(
        username, password,
        case("myPassw0rds", "Passw0rd"),
        case("Passw0rds", "Passw0rd"),
        case("Passw0rd", "Passw0rd"),
        case("myPassw0rd", "Passw0rd"),
        ::trace
    )]
    #[test]
    fn test_validate_password_is_included_in_username(
        username: &'static str,
        password: &'static str,
    )
    {
        run(|_, _, logger| {
            let data = &Json(RequestData {
                username: username.to_string(),
                password: password.to_string(),
            });
            let v = Validator { data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("password", errors[0].field);
                assert_eq!(
                    vec!["Must not overlap with username"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[rstest(
        password, message,
        case("passw0rd", "Must contain 'A-Z'"),
        case("PASSW0RD", "Must contain 'a-z'"),
        case("passworD", "Must contain '0-9'"),
        ::trace
    )]
    #[test]
    fn test_validate_password_is_not_formatted_according_rules(
        password: &'static str,
        message: &'static str,
    )
    {
        run(|_, _, logger| {
            let data = &Json(RequestData {
                username: "username".to_string(),
                password: password.to_string(),
            });
            let v = Validator { data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("password", errors[0].field);
                assert_eq!(vec![message.to_string()], errors[0].messages);
            } else {
                panic!("must fail");
            }
        })
    }
}
