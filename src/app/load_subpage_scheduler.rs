use std::sync::Arc;

use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::events_loop::EventsLoopMutexWrapped;

use crate::{background::RestorePageTask, topics::Topic};

pub struct LoadSubPageScheduler {
    pub restore_page_events_loop: EventsLoopMutexWrapped<RestorePageTask>,
}

impl LoadSubPageScheduler {
    pub fn schedule_load_sub_page(&self, topic: Arc<Topic>, sub_page_id: SubPageId) {
        let task = RestorePageTask { topic, sub_page_id };
        self.restore_page_events_loop.send(task);
    }
}

impl Default for LoadSubPageScheduler {
    fn default() -> Self {
        Self {
            restore_page_events_loop: EventsLoopMutexWrapped::new("RestorePageTasks"),
        }
    }
}
