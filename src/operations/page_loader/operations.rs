use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    time::Duration,
};

use my_service_bus_shared::{
    page_id::PageId,
    sub_page::{SubPage, SubPageId},
    MySbMessageContent,
};

use crate::{
    app::logs::Logs,
    grpc_client::{MessagesPagesRepo, PersistenceError},
    topics::Topic,
};

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
        Some(messages) => SubPage::restore(sub_page_id, messages),
        None => SubPage::restore(sub_page_id, BTreeMap::new()),
    }
}

#[inline]
async fn load_page_from_repo(
    topic: &Topic,
    messages_pages_repo: &Arc<MessagesPagesRepo>,
    logs: Option<&Logs>,
    page_id: PageId,
    sub_page_id: SubPageId,
) -> Option<BTreeMap<i64, MySbMessageContent>> {
    let mut attempt_no = 0;
    loop {
        let result = messages_pages_repo
            .load_page(
                topic.topic_id.as_str(),
                page_id,
                sub_page_id.get_first_message_id(),
                sub_page_id.get_last_message_id(),
            )
            .await;

        if let Ok(result) = result {
            return result;
        }

        let err = result.err().unwrap();
        match err {
            PersistenceError::ZipOperationError(zip_error) => {
                let mut ctx = HashMap::new();

                ctx.insert("pageId".to_string(), page_id.get_value().to_string());
                ctx.insert("attemptNo".to_string(), attempt_no.to_string());
                if let Some(logs) = logs {
                    logs.add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!("Can not load page from persistence storage. Creating empty page. Err:{}", zip_error),
                        Some(ctx),
                    );
                }

                return None;
            }
            _ => {
                if let Some(logs) = logs {
                    let mut ctx = HashMap::new();
                    ctx.insert("pageId".to_string(), page_id.get_value().to_string());
                    ctx.insert("attemptNo".to_string(), attempt_no.to_string());

                    logs.add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                            "Can not load page #{} from persistence storage.Retrying...",
                            page_id.get_value(),
                        ),
                        Some(ctx),
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
