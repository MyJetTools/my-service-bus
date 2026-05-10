use std::{sync::Arc, time::Duration};

use rust_extensions::{AppStates, ApplicationStates};

use crate::{
    grpc_client::PersistenceGrpcService, queue_subscribers::SubscriberIdGenerator,
    sessions::SessionsList, settings::SettingsModel, topics::TopicsList,
    utils::MultiThreadedShortString,
};

use super::{prometheus_metrics::PrometheusMetrics, ImmediatelyPersistEventLoop};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub struct AppContext {
    pub states: Arc<AppStates>,
    pub topic_list: TopicsList,
    pub persistence_client: Arc<PersistenceGrpcService>,
    pub sessions: SessionsList,
    pub subscriber_id_generator: SubscriberIdGenerator,

    pub prometheus: PrometheusMetrics,

    pub delivery_timeout: Duration,

    pub immediately_persist_event_loop: ImmediatelyPersistEventLoop,

    pub persistence_version: MultiThreadedShortString,

    #[cfg(not(test))]
    pub restore_page_scheduler: super::LoadSubPageScheduler,

    #[cfg(test)]
    pub restore_page_scheduler: crate::test_tools::SubPageLoaderSchedulerMock,

    pub settings: Arc<SettingsModel>,
}

impl AppContext {
    pub async fn new(messages_repo: PersistenceGrpcService, settings: Arc<SettingsModel>) -> Self {
        Self {
            states: Arc::new(AppStates::create_un_initialized()),
            topic_list: TopicsList::new(),

            persistence_client: Arc::new(messages_repo),
            sessions: SessionsList::new(),

            subscriber_id_generator: SubscriberIdGenerator::new(),
            prometheus: PrometheusMetrics::new(),

            delivery_timeout: if let Some(delivery_timeout) = settings.delivery_timeout {
                delivery_timeout
            } else {
                Duration::from_secs(30)
            },
            immediately_persist_event_loop: ImmediatelyPersistEventLoop::new(),
            persistence_version: MultiThreadedShortString::new(),

            restore_page_scheduler: Default::default(),
            settings,
        }
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
