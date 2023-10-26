use std::ops::{Deref, DerefMut};

use tokio::sync::MutexGuard;

use super::TopicInner;

pub struct TopicDataAccess<'s> {
    topic_data: MutexGuard<'s, TopicInner>,
}

impl<'s> TopicDataAccess<'s> {
    pub fn new(topic_data: MutexGuard<'s, TopicInner>) -> Self {
        Self { topic_data }
    }
}

impl<'s> Deref for TopicDataAccess<'s> {
    type Target = MutexGuard<'s, TopicInner>;

    fn deref(&self) -> &Self::Target {
        &self.topic_data
    }
}

impl<'s> DerefMut for TopicDataAccess<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.topic_data
    }
}
