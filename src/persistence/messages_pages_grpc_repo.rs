use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use futures_util::stream;

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::page_id::PageId;
use my_service_bus_shared::protobuf_models::MessageProtobufModel;
use my_service_bus_shared::MySbMessageContent;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::persistence_grpc::my_service_bus_messages_persistence_grpc_service_client::MyServiceBusMessagesPersistenceGrpcServiceClient;
use crate::persistence_grpc::*;

use super::protobuf_models::NewMessagesProtobufContract;
use super::PersistenceError;

const PAYLOAD_SIZE: usize = 1024 * 1024 * 4;
pub struct MessagesPagesGrpcRepo {
    channel: Channel,
    time_out: Duration,
}

impl MessagesPagesGrpcRepo {
    pub async fn new(grpc_address: String) -> Self {
        let channel = Channel::from_shared(grpc_address)
            .unwrap()
            .connect()
            .await
            .unwrap();
        Self {
            time_out: Duration::from_secs(5),
            channel,
        }
    }

    fn create_grpc_service(&self) -> MyServiceBusMessagesPersistenceGrpcServiceClient<Channel> {
        MyServiceBusMessagesPersistenceGrpcServiceClient::new(self.channel.clone())
    }

    pub async fn save_messages(
        &self,
        topic_id: &str,
        messages: Vec<MessageProtobufModel>,
    ) -> Result<(), PersistenceError> {
        let grpc_messages = NewMessagesProtobufContract {
            topic_id: topic_id.to_string(),
            messages,
        };

        let grpc_protobuf = grpc_messages.into_protobuf_vec();

        let grpc_protobuf_compressed =
            my_service_bus_shared::page_compressor::zip::compress_payload(
                grpc_protobuf.as_slice(),
            )?;

        let mut grpc_client = self.create_grpc_service();

        let mut grpc_chunks = Vec::new();

        for chunk in split(grpc_protobuf_compressed.as_slice(), PAYLOAD_SIZE) {
            grpc_chunks.push(CompressedMessageChunkModel { chunk });
        }

        let result = tokio::time::timeout(
            self.time_out,
            grpc_client.save_messages(stream::iter(grpc_chunks)),
        )
        .await?;

        if let Err(status) = result {
            return Err(PersistenceError::TonicError(status));
        }

        return Ok(());
    }

    pub async fn get_persistence_version(&self) -> Result<String, PersistenceError> {
        let mut grpc_client = self.create_grpc_service();

        let response = grpc_client.get_version(()).await?.into_inner();
        return Ok(response.version);
    }
    pub async fn save_messages_uncompressed(
        &self,
        topic_id: &str,
        messages: Vec<MessageProtobufModel>,
    ) -> Result<(), PersistenceError> {
        let grpc_messages = NewMessagesProtobufContract {
            topic_id: topic_id.to_string(),
            messages,
        };

        let grpc_protobuf = grpc_messages.into_protobuf_vec();

        let mut grpc_client = self.create_grpc_service();

        let mut grpc_chunks = Vec::new();

        for chunk in split(grpc_protobuf.as_slice(), PAYLOAD_SIZE) {
            grpc_chunks.push(UnCompressedMessageChunkModel { chunk });
        }

        let result = tokio::time::timeout(
            self.time_out,
            grpc_client.save_messages_uncompressed(stream::iter(grpc_chunks)),
        )
        .await?;

        if let Err(status) = result {
            return Err(PersistenceError::TonicError(status));
        }

        return Ok(());
    }

    pub async fn load_page(
        &self,
        topic_id: &str,
        from_message_id: MessageId,
        to_message_id: MessageId,
    ) -> Result<BTreeMap<i64, MySbMessageContent>, PersistenceError> {
        let page_id = PageId::from_message_id(from_message_id);
        let page_id_to = PageId::from_message_id(to_message_id);

        if page_id.get_value() != page_id_to.get_value() {
            panic!(
                "from_message_id {} and to_message_id {} are not in the same page",
                from_message_id, to_message_id
            );
        }

        let mut grpc_client = self.create_grpc_service();

        let mut grpc_stream = tokio::time::timeout(
            self.time_out,
            grpc_client.get_page(GetMessagesPageGrpcRequest {
                topic_id: topic_id.to_string(),
                page_no: page_id.get_value(),
                from_message_id,
                to_message_id,
                version: 1,
            }),
        )
        .await??
        .into_inner();

        let mut messages: BTreeMap<i64, MySbMessageContent> = BTreeMap::new();

        while let Some(stream_result) =
            tokio::time::timeout(self.time_out, grpc_stream.next()).await?
        {
            let grpc_model = stream_result?;
            messages.insert(
                grpc_model.message_id,
                MySbMessageContent {
                    id: grpc_model.message_id,
                    content: grpc_model.data,
                    time: DateTimeAsMicroseconds::new(grpc_model.created),
                    headers: restore_headers(grpc_model.meta_data),
                },
            );
        }

        println!(
            "Read Page {} with messages amount: {}",
            page_id.get_value(),
            messages.len()
        );

        Ok(messages)
    }
}

fn split(src: &[u8], max_payload_size: usize) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = Vec::new();

    let mut pos: usize = 0;

    while pos < src.len() {
        if src.len() - pos < max_payload_size {
            let payload = &src[pos..];
            result.push(payload.to_vec());
            break;
        }
        let payload = &src[pos..pos + max_payload_size];
        result.push(payload.to_vec());
        pos += max_payload_size;
    }

    result
}

fn restore_headers(
    grpc_meta_data: Vec<MessageContentMetaDataItem>,
) -> Option<HashMap<String, String>> {
    if grpc_meta_data.is_empty() {
        return None;
    }

    let mut result = HashMap::new();
    for kv in grpc_meta_data {
        result.insert(kv.key, kv.value);
    }

    Some(result)
}
