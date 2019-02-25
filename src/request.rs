type ID = usize;

#[derive(Deserialize)]
pub struct Message {
    pub id: Option<ID>,
    pub title: String,
    pub description: Option<String>,
}
