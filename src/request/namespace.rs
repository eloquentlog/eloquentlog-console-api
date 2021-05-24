/// Namespace
#[derive(Clone, Deserialize)]
pub struct Namespace {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Default for Namespace {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
        }
    }
}
