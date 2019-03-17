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
            level: Level::Information,
            format: Format::TOML,
            title: input.title,
            content: input.content,
        };

        let result = rules! {
            "code" => m.code => [length_if_present(1, 32)],
            "lang" => m.lang => [either(vec!["en".to_string()])], // default: en
            "level" => m.level => [either(vec![Level::Information])],
            "format" => m.format => [either(vec![Format::TOML])],
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

    use rocket_contrib::json::Json;

    #[test]
    fn test_validate_code_is_empty() {
        let data = Json(Data {
            code: Some("".to_string()), // error
            title: Some("title".to_string()),

            ..Default::default()
        });
        let v = Validator { data };

        let result = v.validate();
        assert!(result.is_err());

        if let Err(errors) = &result {
            assert_eq!("code", errors[0].field);
        } else {
            panic!("must fail");
        }
    }
}
