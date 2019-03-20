use std::result::Result;

use accord::validators::{either, length_if_present};
use rocket_contrib::json::Json;

use validation::{required, max_if_present};
use request::Message as Data;
use model::message::{Format, Level, NewMessage};

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub messages: Vec<String>,
}

pub struct Validator {
    data: Json<Data>,
}

impl Validator {
    pub fn new(data: Json<Data>) -> Self {
        Self { data }
    }

    #[allow(clippy::redundant_closure)]
    pub fn validate(&self) -> Result<Box<NewMessage>, Vec<ValidationError>> {
        let input = self.data.0.clone();

        // TODO: level and format
        let m = NewMessage {
            code: input.code,
            lang: input.lang.unwrap_or_else(|| "en".to_string()),
            level: Level::from(
                input.level.unwrap_or_else(|| "information".to_string()),
            ),
            format: Format::from(
                input.format.unwrap_or_else(|| "toml".to_string()),
            ),
            title: input.title,
            content: input.content,
        };

        let result = rules! {
            "code" => m.code => [length_if_present(1, 32)],
            "lang" => m.lang => [either(vec!["en".to_string()])], // default: en
            "level" => m.level => [either(Level::as_vec())],
            "format" => m.format => [either(Format::as_vec())],
            "title" => m.title => [required(), max_if_present(255)],
            "content" => m.content => [length_if_present(0, 8000)]
        };
        if let Err(v) = result {
            // MultipleError to ValidationError
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

        Ok(Box::new(m))
    }
}

#[cfg(test)]
mod message_test {
    use super::*;

    use std::any::Any;

    use rocket_contrib::json::Json;

    #[test]
    fn test_validate_code_is_empty() {
        let data = Json(Data {
            code: Some("".to_string()),
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

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
    }

    #[test]
    fn test_validate_code_is_too_long() {
        let data = Json(Data {
            code: Some("long".repeat(9).to_string()),
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

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
    }

    #[test]
    fn test_validate_code_is_none() {
        let data = Json(Data {
            code: None,
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_ok());

        if let Ok(m) = result {
            assert!((m as Box<Any>).downcast::<NewMessage>().is_ok());
        } else {
            panic!("must not fail");
        }
    }

    #[test]
    fn test_validate_code() {
        let data = Json(Data {
            code: Some("200".to_string()),
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_ok());

        if let Ok(m) = result {
            assert!((m as Box<Any>).downcast::<NewMessage>().is_ok());
        } else {
            panic!("must not fail");
        }
    }

    #[test]
    fn test_validate_lang_in_invalid() {
        let data = Json(Data {
            lang: Some("unknown".to_string()),
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_err());

        if let Err(errors) = &result {
            assert_eq!(1, errors.len());
            assert_eq!("lang", errors[0].field);
            assert_eq!(vec!["Must be one of , en"], errors[0].messages);
        } else {
            panic!("must fail");
        }
    }

    #[test]
    fn test_validate_lang() {
        let data = Json(Data {
            lang: Some("en".to_string()),
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_ok());

        if let Ok(m) = result {
            assert!((m as Box<Any>).downcast::<NewMessage>().is_ok());
        } else {
            panic!("must not fail");
        }
    }

    // TODO
    // level
    // format

    #[test]
    fn test_validate_title_is_none() {
        let data = Json(Data {
            title: None,

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_err());

        if let Err(errors) = &result {
            assert_eq!(1, errors.len());
            assert_eq!("title", errors[0].field);
            assert_eq!(vec!["Must exist"], errors[0].messages);
        } else {
            panic!("must fail");
        }
    }

    #[test]
    fn test_validate_title_is_too_long() {
        let data = Json(Data {
            title: Some("title".repeat(52).to_string()),

            ..Default::default()
        });
        let v = Validator { data };

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
    }

    #[test]
    fn test_validate_title() {
        let data = Json(Data {
            title: Some("title".repeat(51).to_string()),

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_ok());

        if let Ok(m) = result {
            assert!((m as Box<Any>).downcast::<NewMessage>().is_ok());
        } else {
            panic!("must not fail");
        }
    }

    #[test]
    fn test_validate_content_is_too_long() {
        let data = Json(Data {
            content: Some("text".repeat(2001).to_string()),
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

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
    }

    #[test]
    fn test_validate_content_is_none() {
        let data = Json(Data {
            content: None,
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_ok());

        if let Ok(m) = result {
            assert!((m as Box<Any>).downcast::<NewMessage>().is_ok());
        } else {
            panic!("must not fail");
        }
    }

    #[test]
    fn test_validate_content() {
        let data = Json(Data {
            content: Some("text".repeat(2000).to_string()),
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_ok());

        if let Ok(m) = result {
            assert!((m as Box<Any>).downcast::<NewMessage>().is_ok());
        } else {
            panic!("must not fail");
        }
    }

    #[test]
    fn test_validate_fields_are_default() {
        let data = Json(Data {
            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_err());

        if let Err(errors) = &result {
            assert_eq!(1, errors.len());
            assert_eq!("title", errors[0].field);
            assert_eq!(vec!["Must exist"], errors[0].messages);
        } else {
            panic!("must fail");
        }
    }
}
