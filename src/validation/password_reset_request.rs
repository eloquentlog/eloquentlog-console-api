use std::result::Result;

use accord::validators::{contains, length};
use diesel::PgConnection;
use rocket_contrib::json::Json;

use crate::logger::Logger;
use crate::request::password_reset::PasswordResetRequest as RequestData;
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
    ) -> Self {
        Self { data, logger }
    }

    #[allow(clippy::redundant_closure)]
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let result = rules! {
            // TODO: share this rule with a validation for user registration
            "email" => self.data.0.email => [
                contains("@"),
                contains("."),
                length(6, 128)
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

#[cfg(test)]
mod test {
    use super::*;

    use rocket_contrib::json::Json;

    use crate::model::test::run;

    #[test]
    fn test_validate_email_is_empty() {
        run(|_, _, logger| {
            let data = &Json(RequestData {
                email: "".to_string(),
            });
            let v = Validator { data, logger };

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
        run(|_, _, logger| {
            let data = &Json(RequestData {
                email: "this-is-not-email".to_string(),
            });
            let v = Validator { data, logger };

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
        run(|_, _, logger| {
            let data = &Json(RequestData {
                email: "short".to_string(),
            });
            let v = Validator { data, logger };

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
        run(|_, _, logger| {
            let data = &Json(RequestData {
                email: "long@example.org".repeat(9),
            });
            let v = Validator { data, logger };

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
        run(|_, _, logger| {
            let data = &Json(RequestData {
                email: "long".repeat(33),
            });
            let v = Validator { data, logger };

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
        run(|_, _, logger| {
            let data = &Json(RequestData {
                email: "postmaster@example.org".to_string(),
            });
            let v = Validator { data, logger };

            let result = v.validate();
            assert!(result.is_ok());
        })
    }
}
