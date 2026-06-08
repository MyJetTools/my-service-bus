use std::sync::Arc;

use my_service_bus::abstractions::AsMessageId;
use my_service_bus::shared::sub_page::SubPageId;
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

        // Everything is enqueued unconditionally; here, in the event-loop thread, we cheaply
        // decide under the topic lock whether this restore is still the RIGHT page to load:
        //   - skip if the sub_page is already in cache (an earlier task already loaded it);
        //   - skip if NO queue is currently parked on this sub_page (its cursor already moved
        //     past it) — the task is stale and would restore a page nobody needs anymore.
        // Without the second check a flood of stale duplicates keeps restoring the previous
        // (already-passed) page while the page the cursor actually needs never gets loaded.
        let skip_reason: Option<&str> = {
            let topic_data = model.topic.get_access();

            if topic_data.pages.get_sub_page(model.sub_page_id).is_some() {
                Some("already in cache")
            } else {
                let mut needed = false;
                for queue in topic_data.queues.get_all() {
                    if let Some(peek) = queue.queue.peek() {
                        let cursor_sub_page: SubPageId = peek.as_message_id().into();
                        if cursor_sub_page.get_value() == model.sub_page_id.get_value() {
                            needed = true;
                            break;
                        }
                    }
                }

                if needed {
                    None
                } else {
                    Some("stale: no queue is parked on this sub_page")
                }
            }
        };

        if let Some(reason) = skip_reason {
            if dbg {
                self.app.debug_console.write(format!(
                    "[restore] SKIP topic={} sub_page={} ({})",
                    model.topic.topic_id.as_str(),
                    model.sub_page_id.get_value(),
                    reason
                ));
            }
            return;
        }

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
