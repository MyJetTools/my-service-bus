use std::sync::Arc;

use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus::tcp_contracts::{MySbTcpContract, PacketProtVer};

use crate::http::controllers::MessageToDeliverHttpContract;
use crate::queues::QueueId;
use crate::sessions::SessionType;
use crate::{
    messages_page::MySbMessageContent, queue_subscribers::SubscriberId,
    sessions::MyServiceBusSession, topics::Topic,
};

use super::{SubscriberHttpPackageBuilder, SubscriberTcpPackageBuilder};

pub enum SubscriberPackageBuilderInner {
    Tcp(Option<SubscriberTcpPackageBuilder>),
    Http(Option<SubscriberHttpPackageBuilder>),
}

pub struct SubscriberPackageBuilder {
    pub topic: Arc<Topic>,
    pub queue_id: QueueId,
    pub subscriber_id: SubscriberId,
    pub session: Option<Arc<dyn MyServiceBusSession + Send + Sync + 'static>>,
    inner: SubscriberPackageBuilderInner,
    pub messages_on_delivery: QueueWithIntervals,
}

impl SubscriberPackageBuilder {
    pub fn new(
        session: Arc<dyn MyServiceBusSession + Send + Sync + 'static>,
        topic: Arc<Topic>,
        queue_id: QueueId,
        subscriber_id: SubscriberId,
    ) -> Self {
        match session.get_session_type() {
            SessionType::Tcp(protocol_ver) => {
                Self::create_tcp(topic, queue_id, subscriber_id, session, protocol_ver)
            }
            SessionType::Http => {
                SubscriberPackageBuilder::create_http(topic, queue_id, subscriber_id, session)
            }
            #[cfg(test)]
            SessionType::Test => {
                SubscriberPackageBuilder::create_http(topic, queue_id, subscriber_id, session)
            }
        }
    }

    pub fn create_tcp(
        topic: Arc<Topic>,
        queue_id: QueueId,
        subscriber_id: SubscriberId,
        session: Arc<dyn MyServiceBusSession + Send + Sync + 'static>,
        protocol_ver: PacketProtVer,
    ) -> Self {
        let inner = SubscriberPackageBuilderInner::Tcp(Some(SubscriberTcpPackageBuilder::new(
            &topic,
            &queue_id,
            subscriber_id,
            protocol_ver,
        )));

        Self {
            topic,
            queue_id,
            subscriber_id,
            inner,
            session: Some(session),
            messages_on_delivery: QueueWithIntervals::new(),
        }
    }

    pub fn create_http(
        topic: Arc<Topic>,
        queue_id: QueueId,
        subscriber_id: SubscriberId,
        session: Arc<dyn MyServiceBusSession + Send + Sync + 'static>,
    ) -> Self {
        let inner = SubscriberPackageBuilderInner::Http(Some(SubscriberHttpPackageBuilder::new()));

        Self {
            topic,
            queue_id,
            subscriber_id,
            inner,
            session: Some(session),
            messages_on_delivery: QueueWithIntervals::new(),
        }
    }

    pub fn get_data_size(&self) -> usize {
        match &self.inner {
            SubscriberPackageBuilderInner::Tcp(builder) => {
                builder.as_ref().unwrap().get_data_size()
            }
            SubscriberPackageBuilderInner::Http(builder) => {
                builder.as_ref().unwrap().get_data_size()
            }
        }
    }

    pub fn add_message(&mut self, msg: &MySbMessageContent, attempt_no: i32) {
        let message_id = msg.id.get_value();

        match &mut self.inner {
            SubscriberPackageBuilderInner::Tcp(builder) => {
                builder.as_mut().unwrap().add_message(msg, attempt_no);
            }
            SubscriberPackageBuilderInner::Http(builder) => {
                builder.as_mut().unwrap().add_message(msg, attempt_no);
            }
        }

        self.messages_on_delivery.enqueue(message_id);
    }

    pub fn has_something_to_send(&self) -> bool {
        self.get_data_size() > 0
    }

    pub fn get_tcp_result(&mut self) -> MySbTcpContract {
        match &mut self.inner {
            SubscriberPackageBuilderInner::Tcp(builder) => {
                let builder = builder.take().unwrap();
                builder.get_result()
            }
            SubscriberPackageBuilderInner::Http(_) => {
                panic!("Cannot get tcp result from http package builder");
            }
        }
    }

    pub fn get_http_result(&mut self) -> Vec<MessageToDeliverHttpContract> {
        match &mut self.inner {
            SubscriberPackageBuilderInner::Tcp(_) => {
                panic!("Cannot get http result from tcp package builder");
            }
            SubscriberPackageBuilderInner::Http(builder) => {
                let builder = builder.take().unwrap();
                builder.get_result()
            }
        }
    }

    #[cfg(not(test))]
    pub fn send_messages_to_connection(mut self) {
        if let Some(session) = self.session.take() {
            tokio::task::spawn(async move {
                session.send_messages_to_connection(self).await;
            });
        }
    }

    #[cfg(test)]
    pub async fn send_messages_to_connection(mut self) {
        if let Some(session) = self.session.take() {
            session.send_messages_to_connection(self).await;
        }
    }
}
