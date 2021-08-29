use my_service_bus_shared::{queue_with_intervals::QueueIndexRange, MessageId};

use super::{
    common_serializers::*,
    deserializers::{serialize_long, serialize_message},
    PacketVersions,
};
use crate::{
    app::{AppContext, TEST_QUEUE},
    messages::MySbMessage,
    messages_bucket::MessagesBucket,
    queues::TopicQueueType,
    sessions::PacketProtVer,
    tcp::{MySbSocketError, TSocketReader},
    topics::Topic,
};

use std::collections::HashMap;

pub type RequestId = i64;

pub type ConfirmationId = i64;

const PING: u8 = 0;
const PONG: u8 = 1;
const GREETING: u8 = 2;
const PUBLISH: u8 = 3;
const PUBLISH_RESPONSE: u8 = 4;
const SUBSCRIBE: u8 = 5;
const SUBSCRIBE_RESPONSE: u8 = 6;
pub const NEW_MESSAGE: u8 = 7;
const ALL_MESSAGES_DELIVERED_CONFIRMATION: u8 = 8;
const CREATE_TOPIC_IF_NOT_EXISTS: u8 = 9;
//Not Supported const MESSAGES_DELIVERED_AND_NOT_DELIVERED_CONFIRMATION: u8 = 10;
const PACKET_VERSIONS: u8 = 11;
const REJECT: u8 = 12;
const ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION: u8 = 13;
const CONFIRM_SOME_MESSAGES_AS_OK: u8 = 14;
const CONFIRM_MESSAGES_BY_NOT_DELIVERY: u8 = 15;

#[derive(Debug, Clone)]
pub struct TcpContractMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub content: Vec<u8>,
}

#[derive(Debug)]
pub enum TcpContract {
    Ping,
    Pong,
    Greeting {
        name: String,
        protocol_version: i32,
    },
    Publish {
        topic_id: String,
        request_id: RequestId,
        persist_immediately: bool,
        data_to_publish: Vec<Vec<u8>>,
    },
    PublishResponse {
        request_id: RequestId,
    },
    Subscribe {
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
    },
    SubscribeResponse {
        topic_id: String,
        queue_id: String,
    },
    NewMessages(Vec<u8>),
    NewMessagesConfirmation {
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
    },
    CreateTopicIfNotExists {
        topic_id: String,
    },
    ConfirmMessagesByNotDelivery {
        packet_version: u8,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
        not_delivered: Vec<QueueIndexRange>,
    },
    PacketVersions {
        packet_versions: HashMap<u8, i32>,
    },
    Reject {
        message: String,
    },
    AllMessagesConfirmedAsFail {
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
    },

    ConfirmSomeMessagesAsOk {
        packet_version: u8,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
        delivered: Vec<QueueIndexRange>,
    },
}

pub struct ConnectionAttributes {
    pub versions: PacketVersions,
    pub protocol_version: i32,
}

impl ConnectionAttributes {
    pub fn new() -> Self {
        Self {
            versions: PacketVersions::new(),
            protocol_version: 0,
        }
    }
}

impl TcpContract {
    pub async fn deserialize<T: TSocketReader>(
        socket_reader: &mut T,
        attr: &ConnectionAttributes,
    ) -> Result<TcpContract, MySbSocketError> {
        let packet_no = socket_reader.read_byte().await?;

        let result = match packet_no {
            PING => Ok(TcpContract::Ping {}),
            PONG => Ok(TcpContract::Pong {}),
            GREETING => {
                let name = super::common_deserializers::read_pascal_string(socket_reader).await?;
                let protocol_version = socket_reader.read_i32().await?;

                let result = TcpContract::Greeting {
                    name,
                    protocol_version,
                };
                Ok(result)
            }
            PUBLISH => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let request_id = read_legacy_long(socket_reader, attr).await?;
                let messages_count = socket_reader.read_i32().await? as usize;

                let mut data_to_publish: Vec<Vec<u8>> = Vec::with_capacity(messages_count);

                for _ in 0..messages_count {
                    let byte_array = socket_reader.read_byte_array().await?;
                    data_to_publish.push(byte_array);
                }

                let result = TcpContract::Publish {
                    topic_id,
                    request_id,
                    data_to_publish,
                    persist_immediately: socket_reader.read_bool().await?,
                };
                Ok(result)
            }
            PUBLISH_RESPONSE => {
                let request_id = read_legacy_long(socket_reader, attr).await?;
                let result = TcpContract::PublishResponse { request_id };

                Ok(result)
            }
            SUBSCRIBE => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_type_src = socket_reader.read_byte().await?;

                let queue_type = TopicQueueType::parse(queue_type_src);

                if queue_type.is_none() {
                    return Err(MySbSocketError::InvalidPacket(format!(
                        "Can not parse queuetype with no {} for the Subscribe packet",
                        queue_type_src
                    )));
                }

                let result = TcpContract::Subscribe {
                    topic_id,
                    queue_id,
                    queue_type: queue_type.unwrap(),
                };

                Ok(result)
            }
            SUBSCRIBE_RESPONSE => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let result = TcpContract::SubscribeResponse { topic_id, queue_id };

                Ok(result)
            }
            NEW_MESSAGE => {
                //Client Package
                /*
                let topic_id = socket_reader.read_pascal_string().await?;
                let queue_id = socket_reader.read_pascal_string().await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let records_len = socket_reader.read_i32().await? as usize;
                let packet_version = attr.versions.get_packet_version(packet_no);

                let mut messages: Vec<TcpContractMessage> = Vec::new();
                for _ in 0..records_len {
                    let msg =
                        tcp_packet_message::deserialize(socket_reader, packet_version).await?;
                    messages.push(msg);
                }

                let result = TcpContract::NewMessages {
                    topic_id,
                    queue_id,
                    confirmation_id,
                    messages,
                };
                */

                panic!("This is a client packet. We should not have it on server");
            }
            ALL_MESSAGES_DELIVERED_CONFIRMATION => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let result = TcpContract::NewMessagesConfirmation {
                    topic_id,
                    queue_id,
                    confirmation_id,
                };

                Ok(result)
            }
            CREATE_TOPIC_IF_NOT_EXISTS => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;

                let result = TcpContract::CreateTopicIfNotExists { topic_id };

                Ok(result)
            }

            REJECT => {
                let message =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let result = TcpContract::Reject { message };
                Ok(result)
            }

            PACKET_VERSIONS => {
                let len = socket_reader.read_byte().await?;

                let mut packet_versions: HashMap<u8, i32> = HashMap::new();

                for _ in 0..len {
                    let p = socket_reader.read_byte().await?;
                    let v = socket_reader.read_i32().await?;
                    packet_versions.insert(p, v);
                }

                let result = TcpContract::PacketVersions { packet_versions };

                Ok(result)
            }

            ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION => {
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let result = TcpContract::AllMessagesConfirmedAsFail {
                    topic_id,
                    queue_id,
                    confirmation_id,
                };

                Ok(result)
            }

            CONFIRM_SOME_MESSAGES_AS_OK => {
                let packet_version = socket_reader.read_byte().await?;
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let delivered =
                    super::common_deserializers::read_queue_with_intervals(socket_reader).await?;

                let result = TcpContract::ConfirmSomeMessagesAsOk {
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    delivered,
                };

                Ok(result)
            }

            CONFIRM_MESSAGES_BY_NOT_DELIVERY => {
                let packet_version = socket_reader.read_byte().await?;
                let topic_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let queue_id =
                    super::common_deserializers::read_pascal_string(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let not_delivered =
                    super::common_deserializers::read_queue_with_intervals(socket_reader).await?;

                let result = TcpContract::ConfirmMessagesByNotDelivery {
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    not_delivered,
                };

                Ok(result)
            }

            _ => Err(MySbSocketError::InvalidPacketId(packet_no)),
        };

        return result;
    }

    pub fn serialize(self, attr: &ConnectionAttributes) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        match self {
            TcpContract::Ping {} => {
                result.push(PING);
            }
            TcpContract::Pong {} => {
                result.push(PONG);
            }
            TcpContract::Greeting {
                name,
                protocol_version,
            } => {
                result.push(GREETING);
                serialize_pascal_string(&mut result, name.as_str());
                serialize_i32(&mut result, protocol_version);
            }
            TcpContract::Publish {
                topic_id,
                request_id,
                persist_immediately,
                data_to_publish,
            } => {
                result.push(PUBLISH);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_legacy_long(&mut result, request_id, attr);
                serialize_list_of_arrays(&mut result, &data_to_publish);
                serialize_bool(&mut result, persist_immediately);
            }

            TcpContract::PublishResponse { request_id } => {
                result.push(PUBLISH_RESPONSE);
                serialize_legacy_long(&mut result, request_id, attr);
            }
            TcpContract::Subscribe {
                topic_id,
                queue_id,
                queue_type,
            } => {
                result.push(SUBSCRIBE);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_byte(&mut result, queue_type.into());
            }
            TcpContract::SubscribeResponse { topic_id, queue_id } => {
                result.push(SUBSCRIBE_RESPONSE);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
            }
            TcpContract::NewMessages(payload) => {
                return payload;
            }
            TcpContract::NewMessagesConfirmation {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                result.push(ALL_MESSAGES_DELIVERED_CONFIRMATION);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_i64(&mut result, confirmation_id);
            }
            TcpContract::CreateTopicIfNotExists { topic_id } => {
                result.push(CREATE_TOPIC_IF_NOT_EXISTS);
                serialize_pascal_string(&mut result, topic_id.as_str());
            }
            TcpContract::ConfirmMessagesByNotDelivery {
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                not_delivered,
            } => {
                result.push(CONFIRM_MESSAGES_BY_NOT_DELIVERY);
                result.push(packet_version);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_i64(&mut result, confirmation_id);

                super::common_serializers::serialize_queue_with_intervals(
                    &mut result,
                    &not_delivered,
                );
            }
            TcpContract::PacketVersions { packet_versions } => {
                result.push(PACKET_VERSIONS);

                let data_len = packet_versions.len() as u8;
                serialize_byte(&mut result, data_len);

                for kv in packet_versions {
                    serialize_byte(&mut result, kv.0);
                    serialize_i32(&mut result, kv.1);
                }
            }
            TcpContract::Reject { message } => {
                result.push(REJECT);
                serialize_pascal_string(&mut result, message.as_str());
            }
            TcpContract::AllMessagesConfirmedAsFail {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                result.push(ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_i64(&mut result, confirmation_id);
            }

            TcpContract::ConfirmSomeMessagesAsOk {
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                result.push(CONFIRM_SOME_MESSAGES_AS_OK);
                result.push(packet_version);
                serialize_pascal_string(&mut result, topic_id.as_str());
                serialize_pascal_string(&mut result, queue_id.as_str());
                serialize_i64(&mut result, confirmation_id);

                super::common_serializers::serialize_queue_with_intervals(&mut result, &delivered);
            }
        }

        return result;
    }
}

async fn read_legacy_long<T: TSocketReader>(
    data_reader: &mut T,
    attr: &ConnectionAttributes,
) -> Result<i64, MySbSocketError> {
    if attr.protocol_version >= 2 {
        return data_reader.read_i64().await;
    }

    return match data_reader.read_i32().await {
        Ok(res) => Ok(res as i64),
        Err(err) => Err(err),
    };
}

pub fn serialize_legacy_long(
    data: &mut Vec<u8>,
    request_id: RequestId,
    attr: &ConnectionAttributes,
) {
    if attr.protocol_version < 2 {
        serialize_i32(data, request_id as i32);
    } else {
        serialize_i64(data, request_id);
    }
}

pub async fn compile_messages_delivery_contract(
    app: &AppContext,
    messages_to_deliver: &MessagesBucket,
    topic: &Topic,
    queue_id: &str,
    subscriber_id: i64,
) -> TcpContract {
    let mut result = Vec::new();

    let versions = app
        .sessions
        .get_packet_and_protocol_version(subscriber_id, NEW_MESSAGE)
        .await;

    if queue_id == TEST_QUEUE {
        println!("NEW_MESSAGE Packet version {:?}", versions);
    }

    result.push(NEW_MESSAGE);
    serialize_pascal_string(&mut result, topic.topic_id.as_str());
    serialize_pascal_string(&mut result, queue_id);
    serialize_long(&mut result, subscriber_id, &versions);

    serialize_messages(&mut result, &versions, messages_to_deliver).await;

    TcpContract::NewMessages(result)
}

async fn serialize_messages(
    result: &mut Vec<u8>,
    ver: &PacketProtVer,
    messages_to_deliver: &MessagesBucket,
) {
    let messages_count = messages_to_deliver.messages_count() as i32;

    serialize_i32(result, messages_count);

    for page in &messages_to_deliver.pages {
        let read_access = page.page.data.read().await;

        for msg_id in &page.ids {
            let msg_result = read_access.messages.get(&msg_id);

            if let Some(my_sb_message) = msg_result {
                if let MySbMessage::Loaded(msg) = my_sb_message {
                    let msg_to_send = page.messages.get(&msg_id).unwrap();

                    serialize_message(result, msg, msg_to_send.attempt_no, &ver);
                } else {
                    println!("Message is not loaded. Reason {:?}", my_sb_message)
                }
            } else {
                println!("Message not found to pack {:?}", msg_id)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use async_trait::async_trait;

    struct DataReaderMock {
        data: Vec<u8>,
    }

    impl DataReaderMock {
        pub fn new() -> DataReaderMock {
            DataReaderMock { data: Vec::new() }
        }

        pub fn push(&mut self, data: &[u8]) {
            self.data.extend(data);
        }
    }

    #[async_trait]
    impl TSocketReader for DataReaderMock {
        async fn read_byte(&mut self) -> Result<u8, MySbSocketError> {
            let result = self.data.remove(0);
            Ok(result)
        }

        async fn read_i32(&mut self) -> Result<i32, MySbSocketError> {
            const DATA_SIZE: usize = 4;

            let mut buf = [0u8; DATA_SIZE];

            buf.copy_from_slice(&self.data[0..DATA_SIZE]);

            let result = i32::from_le_bytes(buf);

            for _ in 0..DATA_SIZE {
                self.data.remove(0);
            }

            Ok(result)
        }

        async fn read_bool(&mut self) -> Result<bool, MySbSocketError> {
            let result = self.read_byte().await?;
            Ok(result > 0u8)
        }

        async fn read_byte_array(&mut self) -> Result<Vec<u8>, MySbSocketError> {
            let len = self.read_i32().await? as usize;

            let mut result: Vec<u8> = Vec::new();

            for b in self.data.drain(0..len) {
                result.push(b);
            }

            Ok(result)
        }

        async fn read_buf(&mut self, buf: &mut [u8]) -> Result<(), MySbSocketError> {
            buf.copy_from_slice(self.data.drain(0..buf.len()).as_slice());
            Ok(())
        }

        async fn read_i64(&mut self) -> Result<i64, MySbSocketError> {
            const DATA_SIZE: usize = 8;

            let mut buf = [0u8; DATA_SIZE];

            buf.copy_from_slice(&self.data[0..DATA_SIZE]);

            let result = i64::from_le_bytes(buf);

            for _ in 0..DATA_SIZE {
                self.data.remove(0);
            }

            Ok(result)
        }
    }

    #[tokio::test]
    async fn test_greeting_packet() {
        let test_app_name = "testtttt";
        let test_protocol_version = 255;

        let tcp_packet = TcpContract::Greeting {
            name: test_app_name.to_string(),
            protocol_version: test_protocol_version,
        };

        let mut socket_reader = DataReaderMock::new();

        let attr = ConnectionAttributes::new();

        let serialized_data: Vec<u8> = tcp_packet.serialize(&attr);

        socket_reader.push(&serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Greeting {
                name,
                protocol_version,
            } => {
                assert_eq!(test_app_name, name);
                assert_eq!(test_protocol_version, protocol_version);
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }
}
