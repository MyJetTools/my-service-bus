use std::sync::Arc;

use rust_extensions::events_loop::EventsLoopTick;

use crate::app::AppContext;

use super::RestorePageTask;

pub struct RestoreSubPagesEventLoop {
    pub app: Arc<AppContext>,
}

impl RestoreSubPagesEventLoop {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<RestorePageTask> for RestoreSubPagesEventLoop {
    async fn started(&self) {}
    async fn finished(&self) {}
    async fn tick(&self, model: RestorePageTask) {
        crate::operations::page_loader::load_page_to_cache(
            &model.topic,
            &self.app.persistence_client,
            model.sub_page_id,
        )
        .await;

        let app = self.app.clone();
        let topic = model.topic;
        tokio::spawn(async move {
            let mut topic_access = topic.get_access().await;
            crate::operations::delivery::try_to_deliver_to_subscribers(
                app.as_ref(),
                &topic,
                &mut topic_access,
            );
        });
    }
}
