use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use my_logger::LogEventCtx;
use my_service_bus::shared::sub_page::SubPageId;

use crate::{
    grpc_client::{PersistenceGrpcService, PersistenceError},
    messages_page::{MySbCachedMessage, SubPage, SubPageInner},
    topics::Topic,
};

pub async fn load_page(
    topic: &Topic,
    messages_pages_repo: &Arc<PersistenceGrpcService>,
    sub_page_id: SubPageId,
) -> SubPage {
    let mut attempt_no = 0;
    loop {
        let result = messages_pages_repo
            .load_page(
                topic.topic_id.as_str(),
                sub_page_id.into(),
                sub_page_id.get_first_message_id(),
                sub_page_id.get_last_message_id(),
            )
            .await;

        if let Ok(result) = result {
            match result {
                Some(mut messages) => {
                    let mut result = BTreeMap::new();
                    for message_id in sub_page_id.iterate_message_ids() {
                        if let Some(message) = messages.remove(&message_id) {
                            result.insert(message_id, message.into());
                        } else {
                            result
                                .insert(message_id, MySbCachedMessage::Missing(message_id.into()));
                        }
                    }

                    return SubPageInner::restore(sub_page_id, result).into();
                }
                None => return SubPage::create_as_missing(sub_page_id),
            }
        }

        let err = result.err().unwrap();
        match err {
            PersistenceError::ZipOperationError(zip_error) => {
                my_logger::LOGGER.write_error(
                    "load_page",
                    format!(
                        "Can not load page from persistence storage. Creating empty page. Err:{}",
                        zip_error
                    ),
                    LogEventCtx::new()
                        .add("topicId", topic.topic_id.as_str())
                        .add("subPageId", sub_page_id.get_value().to_string())
                        .add("attemptNo", attempt_no.to_string()),
                );

                return SubPage::create_as_missing(sub_page_id);
            }
            _ => {
                let mut ctx = HashMap::new();
                ctx.insert("subPageId".to_string(), sub_page_id.get_value().to_string());
                ctx.insert("attemptNo".to_string(), attempt_no.to_string());

                my_logger::LOGGER.write_error(
                    "load_page",
                    format!(
                        "Can not load sub_page #{} from persistence storage. Retrying...",
                        sub_page_id.get_value(),
                    ),
                    LogEventCtx::new()
                        .add("topicId", topic.topic_id.as_str())
                        .add("subPageId", sub_page_id.get_value().to_string())
                        .add("attemptNo", attempt_no.to_string()),
                );
            }
        }

        attempt_no += 1;

        if attempt_no == 5 {
            return SubPage::create_as_missing(sub_page_id);
        }
        tokio::time::sleep(Duration::from_secs(1)).await
    }
}
