use std::sync::Arc;

use my_service_bus_shared::page_id::PageId;

use crate::app::AppContext;

pub fn load_page_and_try_to_deliver_again(
    app: &Arc<AppContext>,
    topic: Arc<crate::topics::Topic>,
    page_id: PageId,
    sub_page_id: my_service_bus_shared::sub_page::SubPageId,
) {
    let app = app.clone();

    tokio::spawn(async move {
        crate::operations::page_loader::load_page_to_cache(
            topic.clone(),
            app.messages_pages_repo.clone(),
            Some(app.logs.as_ref()),
            page_id,
            sub_page_id,
        )
        .await;

        let mut topic_data = topic.get_access().await;
        crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);
    });
}
