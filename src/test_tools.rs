use std::sync::{Arc, Mutex};

use my_service_bus::shared::sub_page::SubPageId;

use crate::{app::AppContext, background::RestorePageTask, topics::Topic};

#[derive(Default)]
struct SubpageLoaderInner {
    app: Option<Arc<AppContext>>,
    queue: Vec<RestorePageTask>,
}

#[derive(Default)]
pub struct SubPageLoaderSchedulerMock {
    inner: Mutex<SubpageLoaderInner>,
}

impl SubPageLoaderSchedulerMock {
    pub fn apply_app_ctx(&self, app: Arc<AppContext>) {
        let mut access = self.inner.lock().unwrap();
        access.app = Some(app);
    }
    pub fn schedule_load_sub_page(&self, topic: Arc<Topic>, sub_page_id: SubPageId) {
        println!(
            "Scheduling loading subpage for topic {} with id {}",
            topic.topic_id.as_str(),
            sub_page_id.get_value()
        );

        let mut access = self.inner.lock().unwrap();
        access.queue.push(RestorePageTask { topic, sub_page_id });
    }

    fn dequeue_task(&self) -> Option<(Arc<AppContext>, RestorePageTask)> {
        let mut access = self.inner.lock().unwrap();
        let result = access.queue.pop()?;
        Some((access.app.clone().unwrap(), result))
    }

    pub async fn emulate_event_loop_tick(&self) {
        let Some(task) = self.dequeue_task() else {
            return;
        };

        let (app, task) = task;

        crate::operations::page_loader::load_page_to_cache(
            &task.topic,
            &app.persistence_client,
            task.sub_page_id,
        )
        .await;

        let mut topic_access = task.topic.get_access().await;
        crate::operations::delivery::try_to_deliver_to_subscribers(
            app.as_ref(),
            &task.topic,
            &mut topic_access,
        );
    }
}
