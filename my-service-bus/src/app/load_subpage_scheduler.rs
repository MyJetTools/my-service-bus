use std::sync::Arc;

use my_service_bus::shared::sub_page::SubPageId;

use crate::{background::RestorePageTask, topics::Topic};
use rust_extensions::events_loop::EventsLoop;

pub struct LoadSubPageScheduler {
    pub restore_page_events_loop: EventsLoop<RestorePageTask>,
}

impl LoadSubPageScheduler {
    // We deliberately do NOT dedup here: just enqueue every request. Skipping an unnecessary
    // restore is cheap (a lock + map lookup in the event-loop thread) and is done at the
    // consumer right before the expensive load — see RestoreSubPagesEventLoop::tick, which
    // skips when the page is already cached or no queue still needs it.
    pub fn schedule_load_sub_page(&self, topic: Arc<Topic>, sub_page_id: SubPageId) {
        let task = RestorePageTask { topic, sub_page_id };
        self.restore_page_events_loop.send(task);
    }
}

impl Default for LoadSubPageScheduler {
    fn default() -> Self {
        Self {
            restore_page_events_loop: EventsLoop::new("RestorePageTasks"),
        }
    }
}
