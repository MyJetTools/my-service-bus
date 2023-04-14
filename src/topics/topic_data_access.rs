use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::{Mutex, MutexGuard};

use super::TopicData;

pub struct TopicDataAccess<'s> {
    topic_data: MutexGuard<'s, TopicData>,
    process_taken: Arc<Mutex<Vec<&'static str>>>,
    process: &'static str,
}

impl<'s> TopicDataAccess<'s> {
    pub fn new(
        topic_data: MutexGuard<'s, TopicData>,
        process_taken: Arc<Mutex<Vec<&'static str>>>,
        process: &'static str,
    ) -> Self {
        Self {
            topic_data,
            process_taken,
            process,
        }
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

impl<'s> Drop for TopicDataAccess<'s> {
    fn drop(&mut self) {
        println!("Dropping Access {}", self.topic_data.topic_id);
        let process_taken = self.process_taken.clone();
        let process = self.process.clone();

        tokio::spawn(async move {
            let write_access = self.process_taken.lock().await;

            write_access.retain(|p| p != &process);
        });
    }
}
