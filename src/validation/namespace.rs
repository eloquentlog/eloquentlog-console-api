use std::result::Result;

use accord::validators::{length, length_if_present};
use rocket_contrib::json::Json;

use crate::logger::Logger;
use crate::model::namespace::NewNamespace;
use crate::request::namespace::Namespace as RequestData;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub messages: Vec<String>,
}

pub struct Validator<'a> {
    data: &'a Json<RequestData>,
    _logger: &'a Logger,
}

impl<'a> Validator<'a> {
    pub fn new(data: &'a Json<RequestData>, _logger: &'a Logger) -> Self {
        Self { data, _logger }
    }

    #[allow(clippy::redundant_closure)]
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let n = NewNamespace::from(self.data.0.clone());
        let result = rules! {
            "name" => n.name => [length(3, 255)],
            "description" => n.description => [length_if_present(0, 3000)]
        };
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

    use std::panic::{self, AssertUnwindSafe};

    use dotenv::dotenv;
    use rocket_contrib::json::Json;

    use crate::config::Config;
    use crate::logger::{Logger, get_logger};

    pub fn run<T>(test: T)
    where T: FnOnce(&Logger) + panic::UnwindSafe {
        // TODO: remove dotenv from here
        dotenv().ok();
        let config = Config::from("testing").unwrap();
        let logger = get_logger(&config);

        let result = panic::catch_unwind(AssertUnwindSafe(|| test(&logger)));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_name_is_none() {
        run(|logger| {
            let data = Json(RequestData {
                name: None,

                ..Default::default()
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("name", errors[0].field);
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
    fn test_validate_name_is_too_short() {
        run(|logger| {
            let data = Json(RequestData {
                name: Some("na".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("name", errors[0].field);
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
    fn test_validate_name_is_too_long() {
        run(|logger| {
            let data = Json(RequestData {
                name: Some("name".repeat(64)),

                ..Default::default()
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("name", errors[0].field);
                assert_eq!(
                    vec!["Must contain less than 255 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_name() {
        run(|logger| {
            let data = Json(RequestData {
                name: Some("name".repeat(63)),

                ..Default::default()
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_description_is_too_long() {
        run(|logger| {
            let data = Json(RequestData {
                description: Some("text".repeat(751)),
                name: Some("name".to_string()),
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("description", errors[0].field);
                assert_eq!(
                    vec!["Must contain less than 3000 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_description_is_none() {
        run(|logger| {
            let data = Json(RequestData {
                description: None,
                name: Some("name".to_string()),
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_description() {
        run(|logger| {
            let data = Json(RequestData {
                description: Some("text".repeat(750)),
                name: Some("name".to_string()),
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_fields_are_default() {
        run(|logger| {
            let data = Json(RequestData {
                ..Default::default()
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("name", errors[0].field);
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
    fn test_validate() {
        run(|logger| {
            let data = &Json(RequestData {
                name: Some("changelog".to_string()),
                description: Some(
                    r#"
This is a namespace for testing.
"#
                    .to_string(),
                ),
            });
            let v = Validator::new(&data, &logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }
}
