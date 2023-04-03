use std::sync::Arc;

use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus_shared::MySbMessageContent;
use my_service_bus_tcp_shared::{delivery_package_builder::DeliverTcpPacketBuilder, TcpContract};

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
    pub subscriber_id: SubscriberId,
    tcp_builder: DeliverTcpPacketBuilder,
    data_size: usize,
    session: Arc<MyServiceBusSession>,
    messages_on_delivery: QueueWithIntervals,
}

impl SubscriberPackageBuilder {
    pub fn new(
        topic: Arc<Topic>,
        queue_id: String,
        subscriber_id: SubscriberId,
        session: Arc<MyServiceBusSession>,
    ) -> Self {
        let tcp_builder = DeliverTcpPacketBuilder::new(
            topic.topic_id.as_str(),
            queue_id.as_str(),
            subscriber_id,
            session.get_message_to_delivery_protocol_version(),
        );
        Self {
            topic,
            queue_id,
            tcp_builder,
            subscriber_id,
            data_size: 0,
            session,
            messages_on_delivery: QueueWithIntervals::new(),
        }
    }

    pub fn get_data_size(&self) -> usize {
        self.data_size
    }

    pub fn add_message(&mut self, msg: &MySbMessageContent, attempt_no: i32) {
        self.data_size += msg.content.len();
        self.tcp_builder.append_packet(msg, attempt_no);
    }

    pub fn get_result(self) -> SendNewMessagesResult {
        if self.data_size == 0 {
            return SendNewMessagesResult::NothingToSend {
                queue_id: self.queue_id,
            };
        }

        let tcp_contract = self.tcp_builder.get_result();

        SendNewMessagesResult::Send {
            tcp_contract,
            session: self.session,
            queue_id: self.queue_id,
            messages_on_delivery: self.messages_on_delivery,
        }
    }
}
