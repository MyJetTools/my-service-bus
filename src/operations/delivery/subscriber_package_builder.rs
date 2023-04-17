use std::sync::Arc;

use my_service_bus_abstractions::{queue_with_intervals::QueueWithIntervals, MyServiceBusMessage};

use my_service_bus_tcp_shared::{delivery_package_builder::DeliverTcpPacketBuilder, TcpContract};
use rust_extensions::ShortString;

use crate::{
    messages_page::MySbMessageContent, queue_subscribers::SubscriberId,
    sessions::MyServiceBusSession, topics::Topic,
};

pub struct PacketToSendWrapper<'s> {
    pub attempt: i32,
    pub inner: &'s MySbMessageContent,
}

impl<'s> MyServiceBusMessage for PacketToSendWrapper<'s> {
    fn get_id(&self) -> my_service_bus_abstractions::MessageId {
        self.inner.id
    }

    fn get_attempt_no(&self) -> i32 {
        self.attempt
    }

    fn get_headers(&self) -> Option<&std::collections::HashMap<String, String>> {
        self.inner.headers.as_ref()
    }

    fn get_content(&self) -> &[u8] {
        self.inner.content.as_slice()
    }
}

pub enum SendNewMessagesResult {
    Send {
        session: Arc<MyServiceBusSession>,
        tcp_contract: TcpContract,
        queue_id: ShortString,
        messages_on_delivery: QueueWithIntervals,
    },
    NothingToSend {
        queue_id: ShortString,
    },
}

pub struct SubscriberPackageBuilder {
    pub topic: Arc<Topic>,
    pub queue_id: ShortString,
    pub subscriber_id: SubscriberId,
    tcp_builder: DeliverTcpPacketBuilder,
    data_size: usize,
    session: Arc<MyServiceBusSession>,
    messages_on_delivery: QueueWithIntervals,
}

impl SubscriberPackageBuilder {
    pub fn new(
        topic: Arc<Topic>,
        queue_id: ShortString,
        subscriber_id: SubscriberId,
        session: Arc<MyServiceBusSession>,
    ) -> Self {
        let tcp_builder = DeliverTcpPacketBuilder::new(
            topic.topic_id.as_str(),
            queue_id.as_str(),
            subscriber_id.get_value(),
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

        let message_id = msg.id.get_value();

        let msg = PacketToSendWrapper {
            attempt: attempt_no,
            inner: msg,
        };
        self.tcp_builder.append_packet(&msg);
        self.messages_on_delivery.enqueue(message_id);
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
