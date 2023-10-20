use std::collections::BTreeMap;

use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::{page_id::PageId, protobuf_models::MessageProtobufModel};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{messages_page::MySbMessageContent, settings::SettingsModel};

#[cfg(test)]
use super::MessagesPagesMockRepo;
use super::{MessagesPagesGrpcRepo, PersistenceError};

pub enum MessagesPagesRepo {
    Grpc(MessagesPagesGrpcRepo),
    #[cfg(test)]
    Mock(MessagesPagesMockRepo),
}

impl MessagesPagesRepo {
    pub async fn create_production_instance(settings: &SettingsModel) -> Self {
        Self::Grpc(MessagesPagesGrpcRepo::new(settings.persistence_grpc_url.to_string()).await)
    }

    #[cfg(test)]
    pub fn create_mock_instance() -> Self {
        Self::Mock(MessagesPagesMockRepo::new())
    }

    pub async fn load_page(
        &self,
        topic_id: &str,
        page_id: PageId,
        from_message_id: MessageId,
        to_message_id: MessageId,
    ) -> Result<Option<BTreeMap<i64, MySbMessageContent>>, PersistenceError> {
        match self {
            MessagesPagesRepo::Grpc(repo) => {
                repo.load_page(topic_id, page_id, from_message_id, to_message_id)
                    .await
            }
            #[cfg(test)]
            MessagesPagesRepo::Mock(repo) => {
                repo.load_page(topic_id, from_message_id, to_message_id)
                    .await
            }
        }
    }

    pub async fn save_messages(
        &self,
        topic_id: &str,
        messages: Vec<MessageProtobufModel>,
    ) -> Result<(), PersistenceError> {
        match self {
            MessagesPagesRepo::Grpc(repo) => repo.save_messages(topic_id, messages).await,
            #[cfg(test)]
            MessagesPagesRepo::Mock(repo) => repo.save_messages(topic_id, messages).await,
        }
    }

    pub async fn save_messages_uncompressed(
        &self,
        topic_id: &str,
        messages: Vec<MessageProtobufModel>,
    ) -> Result<(), PersistenceError> {
        match self {
            MessagesPagesRepo::Grpc(repo) => {
                repo.save_messages_uncompressed(topic_id, messages).await
            }
            #[cfg(test)]
            MessagesPagesRepo::Mock(repo) => repo.save_messages(topic_id, messages).await,
        }
    }

    pub async fn get_persistence_version(&self) -> Option<String> {
        let result = match self {
            MessagesPagesRepo::Grpc(repo) => repo.get_persistence_version().await,
            #[cfg(test)]
            MessagesPagesRepo::Mock(_) => Ok("Mock".to_string()),
        };

        match result {
            Ok(result) => Some(result),
            Err(_) => None,
        }
    }

    pub async fn delete_topic(&self, topic_id: &str, hard_delete_moment: DateTimeAsMicroseconds) {
        match self {
            MessagesPagesRepo::Grpc(repo) => repo.delete_topic(topic_id, hard_delete_moment).await,
            #[cfg(test)]
            MessagesPagesRepo::Mock(_) => {
                println!("Delete topic {} is invoked", topic_id);
            }
        }
    }

    pub async fn restore_topic(&self, topic_id: &str) -> bool {
        match self {
            MessagesPagesRepo::Grpc(repo) => repo.restore_topic(topic_id).await,
            #[cfg(test)]
            MessagesPagesRepo::Mock(_) => {
                println!("Restore topic topic {} is invoked", topic_id);
                return true;
            }
        }
    }
}
