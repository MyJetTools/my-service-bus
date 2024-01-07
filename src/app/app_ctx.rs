use std::{sync::Arc, time::Duration};

use rust_extensions::{events_loop::EventsLoop, AppStates, ApplicationStates};
use tokio::sync::RwLock;

use crate::{
    grpc_client::{MessagesPagesRepo, TopicsAndQueuesSnapshotRepo},
    queue_subscribers::SubscriberIdGenerator,
    sessions::SessionsList,
    settings::SettingsModel,
    topics::{Topic, TopicsList},
    utils::MultiThreadedShortString,
};

use super::prometheus_metrics::PrometheusMetrics;

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct DebugTopicAndQueue {
    pub topic_id: String,
    pub queue_id: String,
}

pub struct AppContext {
    pub states: Arc<AppStates>,
    pub topic_list: TopicsList,
    pub topics_and_queues_repo: Arc<TopicsAndQueuesSnapshotRepo>,
    pub messages_pages_repo: Arc<MessagesPagesRepo>,
    pub sessions: SessionsList,
    pub process_id: String,
    pub subscriber_id_generator: SubscriberIdGenerator,

    pub prometheus: PrometheusMetrics,

    pub delivery_timeout: Duration,

    pub debug_topic_and_queue: RwLock<Option<DebugTopicAndQueue>>,

    pub immediately_persist_event_loop: EventsLoop<Arc<Topic>>,

    pub persistence_version: MultiThreadedShortString,

    pub settings: SettingsModel,
}

impl AppContext {
    pub async fn new(settings: SettingsModel) -> Self {
        let topics_and_queues_repo = settings.create_topics_and_queues_snapshot_repo().await;
        let messages_pages_repo = settings.create_messages_pages_repo().await;
        Self {
            states: Arc::new(AppStates::create_un_initialized()),
            topic_list: TopicsList::new(),
            topics_and_queues_repo: Arc::new(topics_and_queues_repo),
            messages_pages_repo: Arc::new(messages_pages_repo),
            sessions: SessionsList::new(),
            process_id: uuid::Uuid::new_v4().to_string(),

            subscriber_id_generator: SubscriberIdGenerator::new(),
            prometheus: PrometheusMetrics::new(),

            delivery_timeout: if let Some(delivery_timeout) = settings.delivery_timeout {
                delivery_timeout
            } else {
                Duration::from_secs(30)
            },
            debug_topic_and_queue: RwLock::new(None),
            immediately_persist_event_loop: EventsLoop::new(
                "ImmediatelyPersist".to_string(),
                my_logger::LOGGER.clone(),
            ),
            persistence_version: MultiThreadedShortString::new(),
            settings,
        }
    }

    pub async fn set_debug_topic_and_queue(&self, topic_id: &str, queue_id: &str) {
        let mut write_access = self.debug_topic_and_queue.write().await;

        *write_access = Some(DebugTopicAndQueue {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
        })
    }

    pub async fn disable_debug_topic_and_queue(&self) {
        let mut write_access = self.debug_topic_and_queue.write().await;

        *write_access = None;
    }

    pub fn get_max_delivery_size(&self) -> usize {
        self.settings.max_delivery_size
    }
}

impl ApplicationStates for AppContext {
    fn is_initialized(&self) -> bool {
        self.states.is_initialized()
    }

    fn is_shutting_down(&self) -> bool {
        self.states.is_shutting_down()
    }
}
