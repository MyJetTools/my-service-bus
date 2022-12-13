use std::{collections::BTreeMap, sync::Arc, time::Duration};

use my_service_bus_shared::sub_page::{SubPage, SubPageId};

use crate::{persistence::MessagesPagesRepo, topics::Topic};

pub async fn load_page(
    topic: &Topic,
    messages_pages_repo: &Arc<MessagesPagesRepo>,
    sub_page_id: SubPageId,
) -> SubPage {
    let mut attempt_no = 0;
    loop {
        let result = messages_pages_repo
            .load_page(
                topic.topic_id.as_str(),
                sub_page_id.get_first_message_id(),
                sub_page_id.get_first_message_id_of_next_sub_page() - 1,
            )
            .await;

        if let Ok(result) = result {
            return SubPage::restore(sub_page_id, result);
        }

        let err = result.err().unwrap();
        match err {
            crate::persistence::PersistenceError::ZipOperationError(zip_error) => {
                crate::LOGS.
                    add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                            "Can not load sub_page #{} from persistence storage. Attempt #{}. Creating empty page. Err: {:?}",
                            sub_page_id.get_value(), attempt_no, zip_error
                        ),
                        None,
                    );

                return SubPage::restore(sub_page_id, BTreeMap::new());
            }
            _ => {
                crate::LOGS.add_error(
                        Some(topic.topic_id.to_string()),
                        crate::app::logs::SystemProcess::Init,
                        "get_page".to_string(),
                        format!(
                            "Can not load sub_page #{} from persistence storage. Attempt #{}. Err: {:?}, Retrying...",
                            sub_page_id.get_value(), attempt_no, err
                        ),
                            None,
                    );
            }
        }

        attempt_no += 1;

        if attempt_no == 5 {
            return SubPage::restore(sub_page_id, BTreeMap::new());
        }
        tokio::time::sleep(Duration::from_secs(1)).await
    }
}
