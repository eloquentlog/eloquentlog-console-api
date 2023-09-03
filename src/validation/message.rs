use std::result::Result;

use accord::validators::{either, length_if_present};
use rocket_contrib::json::Json;

use crate::logger::Logger;
use crate::model::message::{LogFormat, LogLevel, NewMessage};
use crate::request::message::Message as RequestData;
use crate::validation::*;

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
        let m = NewMessage::from(self.data.0.clone());
        let result = rules! {
            "code" => m.code => [length_if_present(1, 32)],
            "lang" => m.lang => [either(vec!["en".to_string()])], // default: en
            "level" => m.level => [either(LogLevel::as_vec())],
            "format" => m.format => [either(LogFormat::as_vec())],
            "title" => m.title => [required(), max_if_present(255)],
            "content" => m.content => [length_if_present(0, 8000)]
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
    fn test_validate_code_is_empty() {
        run(|logger| {
            let data = Json(RequestData {
                code: Some("".to_string()),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("code", errors[0].field);
                assert_eq!(
                    vec!["Must contain more than 1 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_code_is_too_long() {
        run(|logger| {
            let data = Json(RequestData {
                code: Some("long".repeat(9)),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("code", errors[0].field);
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
    fn test_validate_code_is_none() {
        run(|logger| {
            let data = Json(RequestData {
                code: None,
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_code() {
        run(|logger| {
            let data = Json(RequestData {
                code: Some("200".to_string()),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_lang_in_invalid() {
        run(|logger| {
            let data = Json(RequestData {
                lang: Some("unknown".to_string()),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("lang", errors[0].field);
                assert_eq!(vec!["Must be one of , en"], errors[0].messages);
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_lang() {
        run(|logger| {
            let data = Json(RequestData {
                lang: Some("en".to_string()),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_level_is_invalid() {
        run(|logger| {
            let data = Json(RequestData {
                level: Some("unknown".to_string()),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_level_is_none() {
        run(|logger| {
            let data = Json(RequestData {
                level: None,
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_level() {
        run(|logger| {
            let data = Json(RequestData {
                level: Some("debug".to_string()),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validation_format_is_invalid() {
        run(|logger| {
            let data = Json(RequestData {
                format: Some("unknown".to_string()),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validation_format_is_none() {
        run(|logger| {
            let data = Json(RequestData {
                format: None,
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validation_format() {
        run(|logger| {
            let data = Json(RequestData {
                format: Some("TOML".to_string()),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_title_is_none() {
        run(|logger| {
            let data = Json(RequestData {
                title: None,

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("title", errors[0].field);
                assert_eq!(vec!["Must exist"], errors[0].messages);
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_title_is_too_long() {
        run(|logger| {
            let data = Json(RequestData {
                title: Some("title".repeat(52)),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("title", errors[0].field);
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
    fn test_validate_title() {
        run(|logger| {
            let data = Json(RequestData {
                title: Some("title".repeat(51)),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_content_is_too_long() {
        run(|logger| {
            let data = Json(RequestData {
                content: Some("text".repeat(2001)),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("content", errors[0].field);
                assert_eq!(
                    vec!["Must contain less than 8000 characters"],
                    errors[0].messages
                );
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate_content_is_none() {
        run(|logger| {
            let data = Json(RequestData {
                content: None,
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_validate_content() {
        run(|logger| {
            let data = Json(RequestData {
                content: Some("text".repeat(2000)),
                title: Some("title".to_string()),

                ..Default::default()
            });
            let v = Validator::new(&data, logger);

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
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_err());

            if let Err(errors) = &result {
                assert_eq!(1, errors.len());
                assert_eq!("title", errors[0].field);
                assert_eq!(vec!["Must exist"], errors[0].messages);
            } else {
                panic!("must fail");
            }
        })
    }

    #[test]
    fn test_validate() {
        run(|logger| {
            let data = Json(RequestData {
                agent_id: 1,
                agent_type: Some("person".to_string()),
                stream_id: 1,
                code: Some("301".to_string()),
                lang: Some("en".to_string()),
                level: Some("warn".to_string()),
                format: Some("TOML".to_string()),
                title: Some("deprecated method".to_string()),
                content: Some(
                    r#"
[method]
name = "message::Validator::validate()"

[[reason]]
description = "It's deprecated. Use panic!() instead"
"#
                    .to_string(),
                ),
            });
            let v = Validator::new(&data, logger);

            let result = v.validate();
            assert!(result.is_ok());
        })
    }
}
