use std::sync::Arc;

use rust_extensions::events_loop::EventsLoopTick;

use crate::{app::AppContext, topics::Topic};

pub struct ImmediatelyPersistEventLoop {
    app: Arc<AppContext>,
}

impl ImmediatelyPersistEventLoop {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<Arc<Topic>> for ImmediatelyPersistEventLoop {
    async fn started(&self) {}
    async fn tick(&self, topic: Arc<Topic>) {
        crate::operations::persist_topic_messages(&self.app, &topic).await;
    }
    async fn finished(&self) {}
}
