use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicId(Arc<String>);

impl TopicId {
    pub fn new(value: String) -> Self {
        Self(Arc::new(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.as_str().to_string()
    }
}

impl Into<TopicId> for String {
    fn into(self) -> TopicId {
        TopicId::new(self)
    }
}

impl<'s> Into<TopicId> for &'s str {
    fn into(self) -> TopicId {
        TopicId::new(self.to_string())
    }
}

impl<'s> Into<TopicId> for &'s String {
    fn into(self) -> TopicId {
        TopicId::new(self.to_string())
    }
}
