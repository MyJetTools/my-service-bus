use std::sync::Arc;

use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus_shared::MySbMessageContent;
use my_service_bus_tcp_shared::{PacketProtVer, TcpContract};

use crate::{queue_subscribers::SubscriberId, sessions::MyServiceBusSession, topics::Topic};

pub enum SendNewMessagesResult {
    Send {
        session: Arc<MyServiceBusSession>,
        tcp_contract: TcpContract,
        queue_id: String,
        messages_on_delivery: QueueWithIntervals,
    },
    NothingToSend {
        queue_id: String,
    },
}

pub struct SubscriberPackageBuilder {
    pub topic: Arc<Topic>,
    pub queue_id: String,
    pub payload: Option<Vec<u8>>,
    pub session: Arc<MyServiceBusSession>,
    pub subscriber_id: SubscriberId,
    messages_on_delivery: QueueWithIntervals,
    messages_count_position: usize,
    version: PacketProtVer,
}

impl SubscriberPackageBuilder {
    pub fn new(
        topic: Arc<Topic>,
        queue_id: String,
        session: Arc<MyServiceBusSession>,
        subscriber_id: SubscriberId,
        version: PacketProtVer,
    ) -> Self {
        Self {
            topic,
            queue_id,
            payload: None,
            subscriber_id,
            session,
            messages_on_delivery: QueueWithIntervals::new(),
            messages_count_position: 0,
            version,
        }
    }

    fn get_or_create_payload(&mut self) -> &mut Vec<u8> {
        if self.payload.is_none() {
            let mut payload = Vec::new();

            self.messages_count_position =
                my_service_bus_tcp_shared::delivery_package_builder::init_delivery_package(
                    &mut payload,
                    self.topic.topic_id.as_str(),
                    self.queue_id.as_str(),
                    self.subscriber_id,
                );

            self.payload = Some(payload);
        }

        self.payload.as_mut().unwrap()
    }

    pub fn data_size(&self) -> usize {
        match &self.payload {
            Some(payload) => payload.len(),
            None => 0,
        }
    }

    pub fn add_message(&mut self, message_content: &MySbMessageContent, attempt_no: i32) {
        self.messages_on_delivery.enqueue(message_content.id);

        let version = self.version.clone();

        let payload = self.get_or_create_payload();

        my_service_bus_tcp_shared::tcp_serializers::messages_to_deliver::serialize(
            payload,
            message_content,
            attempt_no,
            &version,
        );
    }

    pub fn get_result(self) -> SendNewMessagesResult {
        if let Some(mut payload) = self.payload {
            my_service_bus_tcp_shared::delivery_package_builder::update_amount_of_messages(
                &mut payload,
                self.messages_count_position,
                self.messages_on_delivery.len() as i32,
            );

            return SendNewMessagesResult::Send {
                session: self.session,
                tcp_contract: TcpContract::Raw(payload),
                queue_id: self.queue_id,
                messages_on_delivery: self.messages_on_delivery,
            };
        }

        SendNewMessagesResult::NothingToSend {
            queue_id: self.queue_id,
        }
    }
}
