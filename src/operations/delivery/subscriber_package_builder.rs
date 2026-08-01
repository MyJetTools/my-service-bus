use std::sync::Arc;

use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus::tcp_contracts::MySbTcpContract;
use rust_extensions::base64::IntoBase64;

use crate::http::controllers::{MessageKeyValueJsonModel, MessageToDeliverHttpContract};
use crate::queues::QueueId;
use crate::sessions::http::HttpDeliveryPackage;
use crate::sessions::MyServiceBusSessionInner;
use crate::{
    messages_page::MySbMessageContent, queue_subscribers::SubscriberId,
    sessions::MyServiceBusSession, topics::Topic,
};

use super::SubscriberTcpPackageBuilder;

/*
pub enum SubscriberPackageBuilderInner {
    Tcp(Option<SubscriberTcpPackageBuilder>),
    Http(Option<SubscriberHttpPackageBuilder>),
}
 */
pub struct SubscriberPackageBuilder {
    pub topic: Arc<Topic>,
    pub queue_id: QueueId,
    pub subscriber_id: SubscriberId,
    pub session: Option<MyServiceBusSession>,
    builder: SubscriberTcpPackageBuilder,
    pub messages_on_delivery: QueueWithIntervals,
}

impl SubscriberPackageBuilder {
    pub fn new(
        session: MyServiceBusSession,
        topic: Arc<Topic>,
        queue_id: QueueId,
        subscriber_id: SubscriberId,
    ) -> Self {
        let builder = match &session.inner {
            MyServiceBusSessionInner::Tcp(session) => SubscriberTcpPackageBuilder::new(
                &topic,
                &queue_id,
                subscriber_id,
                session.get_messages_to_deliver_protocol_version(),
            ),
            MyServiceBusSessionInner::Http(_) => {
                SubscriberTcpPackageBuilder::new_last_version(&topic, &queue_id, subscriber_id)
            }
            #[cfg(test)]
            MyServiceBusSessionInner::Test(_) => {
                SubscriberTcpPackageBuilder::new_last_version(&topic, &queue_id, subscriber_id)
            }
        };

        Self {
            topic,
            queue_id,
            subscriber_id,
            builder,
            session: Some(session),
            messages_on_delivery: QueueWithIntervals::new(),
        }
    }

    /*
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
    */
    pub fn get_data_size(&self) -> usize {
        self.builder.get_data_size()
    }

    pub fn add_message(&mut self, msg: &MySbMessageContent, attempt_no: i32) {
        let message_id = msg.id.get_value();
        self.builder.add_message(msg, attempt_no);
        self.messages_on_delivery.enqueue(message_id);
    }

    pub fn has_something_to_send(&self) -> bool {
        self.get_data_size() > 0
    }

    pub fn get_tcp_result(self) -> MySbTcpContract {
        self.builder.get_result()
    }

    pub fn get_http_result(self) -> HttpDeliveryPackage {
        let contract = self.builder.into_new_messages_model();

        HttpDeliveryPackage {
            topic_id: contract.topic_id,
            queue_id: contract.queue_id,
            subscriber_id: contract.confirmation_id.into(),
            messages: contract
                .messages
                .into_iter()
                .map(|msg| MessageToDeliverHttpContract {
                    id: msg.id.get_value(),
                    attempt_no: msg.attempt_no,
                    headers: msg
                        .headers
                        .into_iter()
                        .map(|h| MessageKeyValueJsonModel {
                            key: h.0,
                            value: h.1,
                        })
                        .collect(),
                    content: msg.content.into_base64(),
                })
                .collect(),
        }
    }

    pub fn send_messages_to_connection(mut self) {
        if let Some(session) = self.session.take() {
            session.send_messages_to_connection(self);
        }
    }
}
