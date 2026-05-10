use std::sync::Arc;

use my_service_bus::abstractions::MyServiceBusMessage;
use my_service_bus::abstractions::SbMessageHeaders;

use my_service_bus::tcp_contracts::new_messages_packet_builder::NewMessagesPacketBuilder;
use my_service_bus::tcp_contracts::*;

use crate::queues::QueueId;
use crate::{messages_page::MySbMessageContent, queue_subscribers::SubscriberId, topics::Topic};

pub struct PacketToSendWrapper<'s> {
    pub attempt: i32,
    pub inner: &'s MySbMessageContent,
}

impl<'s> MyServiceBusMessage for PacketToSendWrapper<'s> {
    fn get_id(&self) -> my_service_bus::abstractions::MessageId {
        self.inner.id
    }

    fn get_attempt_no(&self) -> i32 {
        self.attempt
    }

    fn get_headers(&self) -> &SbMessageHeaders {
        &self.inner.headers
    }

    fn get_content(&self) -> &[u8] {
        self.inner.content.as_slice()
    }
}

pub struct SubscriberTcpPackageBuilder {
    tcp_builder: NewMessagesPacketBuilder,
    data_size: usize,
}

impl SubscriberTcpPackageBuilder {
    pub fn new(
        topic: &Arc<Topic>,
        queue_id: &QueueId,
        subscriber_id: SubscriberId,
        protocol_version: PacketProtVer,
    ) -> Self {
        let tcp_builder = NewMessagesPacketBuilder::new(
            topic.topic_id.as_str(),
            queue_id.as_str(),
            subscriber_id.get_value(),
            protocol_version,
        );
        Self {
            tcp_builder,
            data_size: 0,
        }
    }

    pub fn new_last_version(
        topic: &Arc<Topic>,
        queue_id: &QueueId,
        subscriber_id: SubscriberId,
    ) -> Self {
        let tcp_builder = NewMessagesPacketBuilder::new_last_version(
            topic.topic_id.as_str(),
            queue_id.as_str(),
            subscriber_id.get_value(),
        );
        Self {
            tcp_builder,
            data_size: 0,
        }
    }

    pub fn get_data_size(&self) -> usize {
        self.data_size
    }

    pub fn add_message(&mut self, msg: &MySbMessageContent, attempt_no: i32) {
        self.data_size += msg.content.len();

        let msg = PacketToSendWrapper {
            attempt: attempt_no,
            inner: msg,
        };
        self.tcp_builder.append_packet(&msg);
    }

    pub fn get_result(self) -> MySbTcpContract {
        self.tcp_builder.get_result()
    }

    pub fn into_new_messages_model(self) -> NewMessagesModel {
        self.tcp_builder.into_new_messages_model()
    }
}
