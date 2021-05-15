use std::result::Result;

use accord::validators::{contains, length};
use diesel::PgConnection;
use rocket_contrib::json::Json;

use crate::logger::Logger;
use crate::model::user::{NewUser, User};
use crate::request::user::registration::UserRegistration as RequestData;
use crate::validation::*;

pub struct Validator<'a> {
    conn: &'a PgConnection,
    data: &'a Json<RequestData>,
    logger: &'a Logger,
}

impl<'a> Validator<'a> {
    pub fn new(
        conn: &'a PgConnection,
        data: &'a Json<RequestData>,
        logger: &'a Logger,
    ) -> Self {
        Self { conn, data, logger }
    }

    fn validate_email_uniqueness(&self) -> Result<(), ValidationError> {
        if !User::check_email_uniqueness(
            &self.data.0.email,
            self.conn,
            self.logger,
        ) {
            return Err(ValidationError {
                field: "email".to_string(),
                messages: vec!["Already exists".to_string()],
            });
        }
        Ok(())
    }

    fn validate_username_uniqueness(&self) -> Result<(), ValidationError> {
        if !User::check_username_uniqueness(
            &self.data.0.username,
            self.conn,
            self.logger,
        ) {
            return Err(ValidationError {
                field: "username".to_string(),
                messages: vec!["That username is already taken".to_string()],
            });
        }
        Ok(())
    }

    #[allow(clippy::redundant_closure)]
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let u = NewUser::from(&self.data.0);
        // TODO:
        // * check email format
        // * check whether username is reserved or not
        let result = rules! {
            "name" => u.name => [
                max_if_present(64)
            ],
            "username" => u.username => [
                contain_only_alphanumeric_or_underscore(),
                not_contain_only_digits_or_underscore(),
                not_start_with_digits(),
                not_start_with("_"),
                length(3, 32)
            ],
            "email" => u.email => [
                contains("@"),
                contains("."),
                length(6, 128)
            ],
            "password" => self.data.0.password => [
                contain_any(CHARS_LOWER, "a-z"),
                contain_any(CHARS_UPPER, "A-Z"),
                contain_any(DIGITS, "0-9"),
                not_overlap_with("username")(u.username),
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

        if errors.iter().find(|&e| "email" == e.field).is_none() {
            if let Err(e) = self.validate_email_uniqueness() {
                errors.push(e);
            }
        }

        if errors.iter().find(|&e| "username" == e.field).is_none() {
            if let Err(e) = self.validate_username_uniqueness() {
                errors.push(e);
            }
        }

        if !errors.is_empty() {
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
    fn test_validate_email_is_empty() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "".to_string(),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("email", errors[0].field);
                assert_eq!(
                    vec![
                        "Must contain '@'",
                        "Must contain '.'",
                        "Must contain more than 6 characters",
                    ],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_email_is_invalid() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "this-is-not-email".to_string(),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("email", errors[0].field);
                assert_eq!(
                    vec!["Must contain '@'", "Must contain '.'"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_email_is_invalid_and_too_short() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "short".to_string(),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("email", errors[0].field);
                assert_eq!(
                    vec![
                        "Must contain '@'",
                        "Must contain '.'",
                        "Must contain more than 6 characters",
                    ],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_email_is_too_long() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "long@example.org".repeat(9),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("email", errors[0].field);
                assert_eq!(
                    vec!["Must contain less than 128 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_email_is_invalid_and_too_long() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "long".repeat(33),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("email", errors[0].field);
                assert_eq!(
                    vec![
                        "Must contain '@'",
                        "Must contain '.'",
                        "Must contain less than 128 characters"
                    ],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_email() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_name_is_too_long() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                name: Some("long".repeat(26)),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("name", errors[0].field);
                assert_eq!(
                    vec!["Must contain less than 64 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_name_is_none() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                name: None,
                username: "username".to_string(),
                password: "Passw0rd".to_string(),
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_name() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                name: Some("Lorem ipsum".to_string()),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_username_is_too_short() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "hi".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("username", errors[0].field);
                assert_eq!(
                    vec!["Must contain more than 3 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_username_is_too_long() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "username".repeat(5),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("username", errors[0].field);
                assert_eq!(
                    vec!["Must contain less than 32 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[rstest(
        username, message,
        case("-invalid", "Must not contain '-'"),
        case("@invalid", "Must not contain '@'"),
        case("!(-$#@)%", "Must not contain '!'"),
        ::trace
    )]
    #[test]
    fn test_validate_username_is_invalid(
        username: &'static str,
        message: &'static str,
    ) {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: username.to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("username", errors[0].field);
                assert_eq!(vec![message.to_string()], errors[0].messages);
            } else {
                panic!("must fail");
            }
        })
    }

    #[rstest(
        username, messages,
        case(
            "98765432_",
            vec![
                "Must not contain only digits or underscore",
                "Must not start with digits",
            ]
        ),
        case(
            "01234567890",
            vec![
                "Must not contain only digits or underscore",
                "Must not start with digits",
            ]
        ),
        case(
            "122345",
            vec![
                "Must not contain only digits or underscore",
                "Must not start with digits",
            ]
        ),
        ::trace
    )]
    #[test]
    fn test_validate_username_contains_only_digits(
        username: &'static str,
        messages: Vec<&'static str>,
    ) {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: username.to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("username", errors[0].field);

                let error_messages: Vec<_> =
                    messages.iter().map(|m| (*m).to_string()).collect();
                assert_eq!(error_messages, errors[0].messages);
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_username_is_empty() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("username", errors[0].field);
                assert_eq!(
                    vec![
                        "Must not contain only digits or underscore",
                        "Must contain more than 3 characters",
                    ],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[rstest(
        username,
        case("u123456789"),
        case("under_score"),
        case("username009"),
        case("oO0"),
        ::trace
    )]
    #[test]
    fn test_validate_username(username: &'static str) {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: username.to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[rstest(
        username,
        case("0name"),
        case("123four"),
        case("99999a"),
        ::trace
    )]
    #[test]
    fn test_validate_username_starts_with_digits(username: &'static str) {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: username.to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("username", errors[0].field);
                assert_eq!(
                    vec!["Must not start with digits"],
                    errors[0].messages,
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_password_is_too_short() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "username".to_string(),
                password: "Sh0rt".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

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
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "username".to_string(),
                password: "L0ng".repeat(257),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

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
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "Passw0rd".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

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

    #[test]
    fn test_validate_password_contains_username() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "username".to_string(),
                password: "Myusername1sAPartOfpassw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

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

    #[test]
    fn test_validate_password_is_included_in_username() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "myPassw0rd".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

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
    ) {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "username".to_string(),
                password: password.to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

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

    #[test]
    fn test_validate_email_uniqueness() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });

            let mut u = NewUser::from(&data.0);
            u.set_password(&data.password);

            let _id = User::insert(&u, conn, logger)
                .unwrap_or_else(|| panic!("Error inserting: {}", u));

            let data = &Json(RequestData {
                email: u.email,
                username: "newusername".to_string(),
                password: "newPassw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("email", errors[0].field);
                assert_eq!(vec!["Already exists"], errors[0].messages);
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_username_uniqueness() {
        run(|conn, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                username: "username".to_string(),
                password: "Passw0rd".to_string(),

                ..Default::default()
            });

            let mut u = NewUser::from(&data.0);
            u.set_password(&data.password);

            let _id = User::insert(&u, conn, logger)
                .unwrap_or_else(|| panic!("Error inserting: {}", u));

            let data = &Json(RequestData {
                email: "newpostmaster@example.org".to_string(),
                username: u.username,
                password: "newPassw0rd".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data, logger };

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("username", errors[0].field);
                assert_eq!(
                    vec!["That username is already taken"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }
}
