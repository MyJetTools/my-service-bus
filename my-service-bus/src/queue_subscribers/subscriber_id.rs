use std::{fmt::Display, ops::Deref};

#[derive(Debug, Clone, Copy)]
pub struct SubscriberId(i64);

impl SubscriberId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }

    pub fn equals_to(&self, other: SubscriberId) -> bool {
        self.0 == other.0
    }

    pub fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Display for SubscriberId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Into<SubscriberId> for i64 {
    fn into(self) -> SubscriberId {
        SubscriberId::new(self)
    }
}

impl Deref for SubscriberId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
