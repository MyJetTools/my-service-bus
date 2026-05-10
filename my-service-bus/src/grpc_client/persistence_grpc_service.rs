use std::collections::BTreeMap;
use std::sync::Arc;

use my_grpc_extensions::GrpcReadError;
use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::{page_id::PageId, protobuf_models::MessageProtobufModel};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::grpc_client::PersistenceGrpcClient;
use crate::persistence_grpc::*;
use crate::topics::TopicSnapshot;
use crate::{messages_page::MySbMessageContent, settings::SettingsModel};

#[cfg(test)]
use super::MessagesPagesMockRepo;
use super::PersistenceError;

pub enum PersistenceGrpcService {
    Grpc(PersistenceGrpcClient),
    #[cfg(test)]
    Mock(MessagesPagesMockRepo),
}

impl PersistenceGrpcService {
    pub fn create_production_instance(settings: Arc<SettingsModel>) -> Self {
        Self::Grpc(PersistenceGrpcClient::new(settings.clone()))
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
            PersistenceGrpcService::Grpc(repo) => {
                let result = repo
                    .get_page(GetPageGrpcRequest {
                        topic_id: topic_id.to_string(),
                        page_no: page_id.get_value(),
                        from_message_id: from_message_id.get_value(),
                        to_message_id: to_message_id.get_value(),
                        version: 1,
                    })
                    .await
                    .unwrap();

                let result: BTreeMap<i64, MySbMessageContent> = result
                    .into_b_tree_map(|itm| (itm.message_id, itm.into()))
                    .await?;

                Ok(Some(result))
                //  repo.load_page(topic_id, page_id, from_message_id, to_message_id)
                //    .await
            }
            #[cfg(test)]
            PersistenceGrpcService::Mock(repo) => {
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
            PersistenceGrpcService::Grpc(repo) => {
                let input_data = vec![SaveMessagesGrpcRequest {
                    topic_id: topic_id.to_string(),
                    messages: messages.into_iter().map(|itm| itm.into()).collect(),
                }];
                repo.save_messages(input_data).await?;

                Ok(())
            }
            #[cfg(test)]
            PersistenceGrpcService::Mock(repo) => repo.save_messages(topic_id, messages).await,
        }
    }

    pub async fn get_persistence_version(&self) -> Option<String> {
        match self {
            PersistenceGrpcService::Grpc(repo) => match repo.get_version(()).await {
                Ok(result) => Some(result.version),
                Err(_) => None,
            },
            #[cfg(test)]
            PersistenceGrpcService::Mock(_) => Some("Mock".to_string()),
        }
    }

    pub async fn delete_topic(&self, topic_id: &str, hard_delete_moment: DateTimeAsMicroseconds) {
        match self {
            PersistenceGrpcService::Grpc(repo) => {
                repo.delete_topic(DeleteTopicGrpcRequest {
                    topic_id: topic_id.to_string(),
                    delete_after: hard_delete_moment.unix_microseconds,
                })
                .await
                .unwrap();
            }
            #[cfg(test)]
            PersistenceGrpcService::Mock(_) => {
                println!("Delete topic {} is invoked", topic_id);
            }
        }
    }

    pub async fn restore_topic(&self, _topic_id: &str) -> Option<MessageId> {
        match self {
            PersistenceGrpcService::Grpc(_) => {
                todo!("Not Implemented yet");
            }
            #[cfg(test)]
            PersistenceGrpcService::Mock(_) => {
                panic!("Restore topic topic {} is invoked", _topic_id);
            }
        }
    }

    pub async fn get_queue_snapshot(&self) -> Result<Vec<TopicSnapshot>, GrpcReadError> {
        match self {
            PersistenceGrpcService::Grpc(repo) => {
                let result = repo.get_queue_snapshot(()).await?;
                let result = result.into_vec().await?;
                Ok(result)
            }
            #[cfg(test)]
            PersistenceGrpcService::Mock(_) => Ok(vec![]),
        }
    }

    pub async fn save_topic_and_queues(
        &self,
        data: Vec<TopicAndQueuesSnapshotGrpcModel>,
    ) -> Result<(), GrpcReadError> {
        match self {
            PersistenceGrpcService::Grpc(repo) => repo.save_queue_snapshot(data).await,
            #[cfg(test)]
            PersistenceGrpcService::Mock(_) => Ok(()),
        }
    }
}
