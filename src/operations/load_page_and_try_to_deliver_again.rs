use std::sync::Arc;

use my_service_bus::shared::sub_page::SubPageId;

use crate::app::AppContext;

pub fn load_page_and_try_to_deliver_again(
    app: &Arc<AppContext>,
    topic: Arc<crate::topics::Topic>,
    sub_page_id: SubPageId,
    delete_page: bool,
) {
    let app = app.clone();

    tokio::spawn(async move {
        if delete_page {
            let mut topic_data = topic.get_access().await;
            topic_data.pages.delete_sub_page(sub_page_id);
        }

        let sub_page = crate::operations::page_loader::load_page_to_cache(
            &topic,
            app.messages_pages_repo.clone(),
            sub_page_id,
        )
        .await;

        let mut topic_data = topic.get_access().await;

        if let Some(sub_page) = sub_page {
            topic_data.pages.restore_sub_page(sub_page);
        }

        crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);
    });
}
