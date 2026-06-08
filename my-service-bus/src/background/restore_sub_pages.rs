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
        // DEBUG: trace page restore for the topic selected via /api/Debug/Console/Target
        let dbg = self
            .app
            .debug_console
            .matches_topic(model.topic.topic_id.as_str());

        if dbg {
            self.app.debug_console.write(format!(
                "[restore] START topic={} sub_page={}",
                model.topic.topic_id.as_str(),
                model.sub_page_id.get_value()
            ));
        }

        crate::operations::page_loader::load_page_to_cache(
            &model.topic,
            &self.app.persistence_client,
            model.sub_page_id,
        )
        .await;

        if dbg {
            let result = {
                let topic_data = model.topic.get_access();
                match topic_data.pages.get_sub_page(model.sub_page_id) {
                    Some(sub_page) => {
                        let (loaded, missing) = sub_page.get_loaded_and_missing();
                        format!("loaded={loaded} missing={missing}")
                    }
                    None => "NOT in cache after load".to_string(),
                }
            };
            self.app.debug_console.write(format!(
                "[restore] DONE topic={} sub_page={} -> {}",
                model.topic.topic_id.as_str(),
                model.sub_page_id.get_value(),
                result
            ));
        }

        let app = self.app.clone();
        let topic = model.topic;
        tokio::spawn(async move {
            let mut topic_access = topic.get_access();
            crate::operations::delivery::try_to_deliver_to_subscribers(
                app.as_ref(),
                &topic,
                &mut topic_access,
            );
        });
    }
}
