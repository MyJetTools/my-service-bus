use std::{collections::BTreeMap, sync::Arc, time::Duration};

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::{
    page_id::PageId,
    sub_page::{SubPage, SubPageId},
    MySbMessageContent,
};

use crate::{app::logs::Logs, persistence::MessagesPagesRepo, topics::Topic};

pub async fn load_page(
    topic: &Topic,
    messages_pages_repo: &Arc<MessagesPagesRepo>,
    logs: Option<&Logs>,
    page_id: PageId,
    sub_page_id: SubPageId,
) -> SubPage {
    let messages =
        load_page_from_repo(topic, messages_pages_repo, logs, page_id, sub_page_id).await;

    match messages {
        Some(messages) => SubPage::restored(sub_page_id, messages),
        None => SubPage::restored(sub_page_id, BTreeMap::new()),
    }
}

#[inline]
async fn load_page_from_repo(
    topic: &Topic,
    messages_pages_repo: &Arc<MessagesPagesRepo>,
    logs: Option<&Logs>,
    page_id: PageId,
    sub_page_id: SubPageId,
) -> Option<BTreeMap<MessageId, MySbMessageContent>> {
    let mut attempt_no = 0;
    loop {
        let result = messages_pages_repo
            .load_page(
                topic.topic_id.as_str(),
                page_id,
                sub_page_id.get_first_message_id(),
                sub_page_id.get_first_message_id_of_next_sub_page() - 1,
            )
            .await;

        if let Ok(result) = result {
            return result;
        }

        let err = result.err().unwrap();
        match err {
            crate::persistence::PersistenceError::ZipOperationError(zip_error) => {
                if let Some(logs) = logs {
                    logs
                    .add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                            "Can not load page #{} from persistence storage. Attempt #{}. Creating empty page. Err: {:?}",
                            page_id, attempt_no, zip_error
                        ),
                        None,
                    );
                }

                return None;
            }
            _ => {
                if let Some(logs) = logs {
                    logs.add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                            "Can not load page #{} from persistence storage. Attempt #{}. Err: {:?}, Retrying...",
                            page_id, attempt_no, err
                        ),
                            None,
                    );
                }
            }
        }

        attempt_no += 1;

        if attempt_no == 5 {
            return None;
        }
        tokio::time::sleep(Duration::from_secs(1)).await
    }
}
