use std::result::Result;

use accord::validators::{length_if_present, max};
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
            "email" => u.email => [max(255)],
            "password" => self.data.0.password => [max(255)]
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
