use std::sync::Arc;

use rust_extensions::events_loop::EventsLoopTick;

use crate::{app::AppContext, topics::Topic};

pub struct ImmediatlyPersistEventLoop {
    app: Arc<AppContext>,
}

impl ImmediatlyPersistEventLoop {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<Arc<Topic>> for ImmediatlyPersistEventLoop {
    async fn tick(&self, topic: Arc<Topic>) {
        crate::operations::save_messages_for_topic(&self.app, &topic).await;
        topic
            .immediatelly_persist_is_charged
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
