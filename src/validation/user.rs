use std::result::Result;

use accord::validators::{contains, length, length_if_present, max, min};
use diesel::PgConnection;
use rocket_contrib::json::Json;

use validation::{
    alphanumeric_underscore_if_present,
    max_if_present,
    not_contain_only_digits_or_underscore_if_present,
    not_overlap_with,
    not_start_with_digits_if_present,
    not_start_with_if_present,
};
use request::User as RequestData;
use model::user::{NewUser, User};

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub messages: Vec<String>,
}

pub struct Validator<'a> {
    conn: &'a PgConnection,
    data: &'a Json<RequestData>,
}

impl<'a> Validator<'a> {
    pub fn new(conn: &'a PgConnection, data: &'a Json<RequestData>) -> Self {
        Self { conn, data }
    }

    fn validate_email_uniqueness(&self) -> Result<(), ValidationError> {
        if !User::check_email_uniqueness(&self.data.0.email, self.conn) {
            return Err(ValidationError {
                field: "email".to_string(),
                messages: vec!["Already exists".to_string()],
            });
        }
        Ok(())
    }

    #[allow(clippy::redundant_closure)]
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let u = NewUser::from(self.data.0.clone());
        // TODO:
        // * email format
        // * (reserved) username
        // * uniqueness (username)
        // * password format
        let result = rules! {
            "name" => u.name => [max_if_present(64)],
            "username" => u.username => [
                alphanumeric_underscore_if_present(),
                length_if_present(3, 32),
                not_contain_only_digits_or_underscore_if_present(),
                not_start_with_digits_if_present(),
                not_start_with_if_present("_")
            ],
            "email" => u.email => [
                contains("@"),
                contains("."),
                length(6, 128)
            ],
            "password" => self.data.0.password => [
                min(8),
                max(1024),
                not_overlap_with("username")(u.username)
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

        if let Err(e) = self.validate_email_uniqueness() {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use rocket_contrib::json::Json;

    use model::test::run;

    #[test]
    fn test_validate_email_is_empty() {
        run(|conn| {
            let data = &Json(RequestData {
                email: "".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                email: "this-is-not-email".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                email: "short".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                email: "long@example.org".repeat(9).to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                email: "long".repeat(33).to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_name_is_too_long() {
        run(|conn| {
            let data = &Json(RequestData {
                name: Some("long".repeat(26).to_string()),
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                name: None,
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_name() {
        run(|conn| {
            let data = &Json(RequestData {
                name: Some("Lorem ipsum".to_string()),
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_username_is_too_short() {
        run(|conn| {
            let data = &Json(RequestData {
                username: Some("hi".to_string()),
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                username: Some("username".repeat(5).to_string()),
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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

    #[test]
    fn test_validate_username_is_invalid() {
        run(|conn| {
            let tests: [(&'static str, &'static str); 3] = [
                ("-invalid", "Must not contain '-'"),
                ("@invalid", "Must not contain '@'"),
                ("!(-$#@)%", "Must not contain '!'"),
            ];

            for (i, (value, message)) in tests.iter().enumerate() {
                let data = &Json(RequestData {
                    username: Some(value.to_string()),
                    email: "postmaster@example.org".to_string(),
                    password: "password".to_string(),

                    ..Default::default()
                });
                let v = Validator { conn, data };

                let result = v.validate();
                assert!(result.is_err());

                if let Err(errors) = &result {
                    assert_eq!(1, errors.len());
                    assert_eq!("username", errors[0].field);
                    assert_eq!(
                        vec![message.to_string()],
                        errors[0].messages,
                        "#{} username: {}",
                        i,
                        value
                    );
                } else {
                    panic!("must fail");
                }
            }
        })
    }

    #[test]
    fn test_validate_username_contains_only_digits() {
        run(|conn| {
            let tests: [(&'static str, Vec<String>); 3] = [
                (
                    "98765432_",
                    vec![
                        "Must not contain only digits or underscore"
                            .to_string(),
                        "Must not start with digits".to_string(),
                    ],
                ),
                (
                    "01234567890",
                    vec![
                        "Must not contain only digits or underscore"
                            .to_string(),
                        "Must not start with digits".to_string(),
                    ],
                ),
                (
                    "122345",
                    vec![
                        "Must not contain only digits or underscore"
                            .to_string(),
                        "Must not start with digits".to_string(),
                    ],
                ),
            ];

            for (i, (value, messages)) in tests.iter().enumerate() {
                let data = &Json(RequestData {
                    username: Some(value.to_string()),
                    email: "postmaster@example.org".to_string(),
                    password: "password".to_string(),

                    ..Default::default()
                });
                let v = Validator { conn, data };

                let result = v.validate();
                assert!(result.is_err());

                if let Err(errors) = &result {
                    dbg!(errors);
                    assert_eq!(1, errors.len());
                    assert_eq!("username", errors[0].field);
                    assert_eq!(
                        messages, &errors[0].messages,
                        "#{} username: {}",
                        i, value
                    );
                } else {
                    panic!("must fail");
                }
            }
        })
    }

    #[test]
    fn test_validate_username_is_none() {
        run(|conn| {
            let data = &Json(RequestData {
                username: None,
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_username() {
        run(|conn| {
            let tests: [&str; 4] =
                ["u123456789", "under_score", "username009", "oO0"];

            for value in tests.iter() {
                let data = &Json(RequestData {
                    username: Some(value.to_string()),
                    email: "postmaster@example.org".to_string(),
                    password: "password".to_string(),

                    ..Default::default()
                });
                let v = Validator { conn, data };

                let result = v.validate();
                assert!(result.is_ok());
            }
        })
    }

    #[test]
    fn test_validate_username_starts_with_digits() {
        run(|conn| {
            let tests: [&'static str; 3] = ["0name", "123four", "99999a"];

            for (i, value) in tests.iter().enumerate() {
                let data = &Json(RequestData {
                    username: Some(value.to_string()),
                    email: "postmaster@example.org".to_string(),
                    password: "password".to_string(),

                    ..Default::default()
                });
                let v = Validator { conn, data };

                let result = v.validate();
                assert!(result.is_err());

                if let Err(errors) = &result {
                    assert_eq!(1, errors.len());
                    assert_eq!("username", errors[0].field);
                    assert_eq!(
                        vec!["Must not start with digits"],
                        errors[0].messages,
                        "#{} username: {}",
                        i,
                        value
                    );
                } else {
                    panic!("must fail");
                }
            }
        })
    }

    #[test]
    fn test_validate_password_is_too_short() {
        run(|conn| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                password: "short".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
                password: "long".repeat(257).to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                username: Some("password".to_string()),
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                username: Some("username".to_string()),
                email: "postmaster@example.org".to_string(),
                password: "myusernameispassword".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
        run(|conn| {
            let data = &Json(RequestData {
                username: Some("mypassword".to_string()),
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
    fn test_validate_email_uniqueness() {
        run(|conn| {
            let data = &Json(RequestData {
                username: Some("username".to_string()),
                email: "postmaster@example.org".to_string(),
                password: "password".to_string(),

                ..Default::default()
            });

            let mut u = NewUser::from(data.0.clone());
            u.set_password(&data.password);

            let _id = User::insert(&u, conn)
                .unwrap_or_else(|| panic!("Error inserting: {}", u));

            let data = &Json(RequestData {
                username: Some("newusername".to_string()),
                email: u.email.to_string(),
                password: "newpassword".to_string(),

                ..Default::default()
            });
            let v = Validator { conn, data };

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
}
