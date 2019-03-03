type ID = usize;

#[derive(Deserialize)]
pub struct Message {
    pub id: Option<ID>,
    pub code: Option<String>,
    pub lang: Option<String>,
    pub level: Option<String>,
    pub format: Option<String>,
    pub title: String,
    pub content: Option<String>,
}
