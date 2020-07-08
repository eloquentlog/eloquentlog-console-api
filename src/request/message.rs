/// Message
#[derive(Clone, Deserialize)]
pub struct Message {
    pub agent_id: i64,
    pub agent_type: Option<String>,
    pub stream_id: i64,
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
            agent_id: -1,
            agent_type: None,
            stream_id: -1,
            code: None,
            lang: None,
            level: None,
            format: None,
            title: None,
            content: None,
        }
    }
}
