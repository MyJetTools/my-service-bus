use std::sync::Arc;

use rust_extensions::MyTimerTick;
use tokio::sync::Mutex;

use crate::{app::AppContext, topics::ReusableTopicsList};

pub struct PersistTopicsAndQueuesTimer {
    app: Arc<AppContext>,
    reusable_topics_vec: Mutex<Option<ReusableTopicsList>>,
}

impl PersistTopicsAndQueuesTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self {
            app,
            reusable_topics_vec: Mutex::new(None),
        }
    }

    async fn get_reusable_topics_vec(&self) -> ReusableTopicsList {
        let mut result = self.reusable_topics_vec.lock().await;

        match result.take() {
            Some(topics) => topics,
            None => ReusableTopicsList::new(),
        }
    }

    async fn put_reusable_topics_vec_back(&self, topics: ReusableTopicsList) {
        let mut result = self.reusable_topics_vec.lock().await;
        *result = Some(topics);
    }
}

#[async_trait::async_trait]
impl MyTimerTick for PersistTopicsAndQueuesTimer {
    async fn tick(&self) {
        let mut reusable_topics = self.get_reusable_topics_vec().await;
        crate::operations::persist_topics_and_queues(&self.app, &mut reusable_topics).await;
        self.put_reusable_topics_vec_back(reusable_topics).await;
    }
}
