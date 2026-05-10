use std::sync::Arc;

use rust_extensions::{
    events_loop::{EventsLoop, EventsLoopPublisher, EventsLoopTick},
    ApplicationStates,
};

use crate::topics::Topic;

pub struct ImmediatelyPersistEventLoop {
    events_loop: EventsLoop<Arc<Topic>>,
    publisher: EventsLoopPublisher<Arc<Topic>>,
}

impl ImmediatelyPersistEventLoop {
    pub fn new() -> Self {
        let  events_loop =
            EventsLoop::new("ImmediatePersist".to_string());
        let publisher = events_loop.get_publisher();
        Self {
            events_loop,
            publisher,
        }
    }

    pub async fn register_event_loop(
        &self,
        events_loop: Arc<dyn EventsLoopTick<Arc<Topic>> + Send + Sync + 'static>,
    ) {
        self.events_loop.register_event_loop(events_loop);
    }

    pub async fn start(&self, app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>) {
        self.events_loop.start( app_states, my_logger::LOGGER.clone());
    }

    pub fn send(&self, topic: Arc<Topic>) {
        self.publisher.send(topic);
    }
}
