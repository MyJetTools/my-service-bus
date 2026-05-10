use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionId(i64);

impl SessionId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }

    pub fn is_eq_to(&self, other: SessionId) -> bool {
        self.0 == other.0
    }

    pub fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Into<SessionId> for i64 {
    fn into(self) -> SessionId {
        SessionId::new(self)
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
