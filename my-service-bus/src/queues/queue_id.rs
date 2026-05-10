use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueueId(Arc<String>);

impl QueueId {
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

impl Into<QueueId> for String {
    fn into(self) -> QueueId {
        QueueId::new(self)
    }
}

impl<'s> Into<QueueId> for &'s str {
    fn into(self) -> QueueId {
        QueueId::new(self.to_string())
    }
}

impl<'s> Into<QueueId> for &'s String {
    fn into(self) -> QueueId {
        QueueId::new(self.to_string())
    }
}
