use crate::request::ID;

/// Message
#[derive(Clone, Deserialize)]
pub struct Message {
    pub id: Option<ID>,
    pub code: Option<String>,
    pub lang: Option<String>,
    pub level: Option<String>,
    pub format: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: None,
            code: None,
            lang: None,
            level: None,
            format: None,
            title: None,
            content: None,
        }
    }
}
