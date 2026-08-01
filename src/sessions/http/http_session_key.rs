#[derive(Debug, Clone)]
pub struct HttpSessionKey(String);

impl HttpSessionKey {
    pub fn new() -> Self {
        let id = uuid::Uuid::new_v4();
        Self(id.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}
