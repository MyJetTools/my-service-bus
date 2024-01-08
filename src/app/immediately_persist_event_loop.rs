use std::sync::Arc;

use rust_extensions::{
    events_loop::{EventsLoop, EventsLoopPublisher, EventsLoopTick},
    ApplicationStates,
};
use tokio::sync::Mutex;

use crate::topics::Topic;

pub struct ImmediatelyPersistEventLoop {
    events_loop: Mutex<EventsLoop<Arc<Topic>>>,
    publisher: EventsLoopPublisher<Arc<Topic>>,
}

impl ImmediatelyPersistEventLoop {
    pub fn new() -> Self {
        let mut events_loop =
            EventsLoop::new("ImmediatePersist".to_string(), my_logger::LOGGER.clone());
        let publisher = events_loop.get_publisher();
        Self {
            events_loop: Mutex::new(events_loop),
            publisher,
        }
    }

    pub async fn register_event_loop(
        &self,
        events_loop: Arc<dyn EventsLoopTick<Arc<Topic>> + Send + Sync + 'static>,
    ) {
        let mut write_access = self.events_loop.lock().await;
        write_access.register_event_loop(events_loop);
    }

    pub async fn start(&self, app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>) {
        let mut write_access = self.events_loop.lock().await;
        write_access.start(app_states);
    }

    pub fn send(&self, topic: Arc<Topic>) {
        self.publisher.send(topic);
    }
}
