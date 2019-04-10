use std::result::Result;

use accord::validators::{contains, length_if_present, max, min};
use rocket_contrib::json::Json;

use validation::max_if_present;
use request::User as RequestData;
use model::user::NewUser;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub messages: Vec<String>,
}

pub struct Validator<'a> {
    data: &'a Json<RequestData>,
}

impl<'a> Validator<'a> {
    pub fn new(data: &'a Json<RequestData>) -> Self {
        Self { data }
    }

    #[allow(clippy::redundant_closure)]
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let u = NewUser::from(self.data.0.clone());
        let result = rules! {
            "name" => u.name => [max_if_present(64)],
            "username" => u.username => [length_if_present(3, 32)],
            "email" => u.email => [contains("@"), contains("."), max(128)],
            "password" => self.data.0.password => [min(8), max(255)]
        };
        // TODO: check uniqueness
        if let Err(v) = result {
            // MultipleError to Vec<ValidationError>
            let errors =
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
            return Err(errors);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use rocket_contrib::json::Json;

    #[test]
    fn test_validate_email_is_empty() {
        let data = &Json(RequestData {
            email: "".to_string(),
            password: "password".to_string(),

            ..Default::default()
        });
        let v = Validator { data };

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
    }

    #[test]
    fn test_validate_email_is_invalid() {
        let data = &Json(RequestData {
            email: "this-is-not-email".to_string(),
            password: "password".to_string(),

            ..Default::default()
        });
        let v = Validator { data };

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
    }

    #[test]
    fn test_validate_email_is_too_long() {
        let data = &Json(RequestData {
            email: "long@example.org".repeat(9).to_string(),
            password: "password".to_string(),

            ..Default::default()
        });
        let v = Validator { data };

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
    }

    #[test]
    fn test_validate_email_is_invalid_and_too_long() {
        let data = &Json(RequestData {
            email: "long".repeat(33).to_string(),
            password: "password".to_string(),

            ..Default::default()
        });
        let v = Validator { data };

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
    }
}
