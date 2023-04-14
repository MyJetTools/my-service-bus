use std::ops::{Deref, DerefMut};

use tokio::sync::MutexGuard;

use super::TopicData;

pub struct TopicDataAccess<'s> {
    topic_data: MutexGuard<'s, TopicData>,
}

impl<'s> TopicDataAccess<'s> {
    pub fn new(topic_data: MutexGuard<'s, TopicData>) -> Self {
        Self { topic_data }
    }
}

impl<'s> Deref for TopicDataAccess<'s> {
    type Target = MutexGuard<'s, TopicData>;

    fn deref(&self) -> &Self::Target {
        &self.topic_data
    }
}

impl<'s> DerefMut for TopicDataAccess<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.topic_data
    }
}
