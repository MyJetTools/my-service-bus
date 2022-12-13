use std::sync::Arc;

use my_service_bus_shared::sub_page::SubPageId;

use crate::app::AppContext;

pub fn load_page_and_try_to_deliver_again(
    app: &Arc<AppContext>,
    topic: Arc<crate::topics::Topic>,
    sub_page_id: SubPageId,
) {
    let app = app.clone();

    tokio::spawn(async move {
        crate::operations::page_loader::load_page_to_cache(
            topic.clone(),
            app.messages_pages_repo.clone(),
            sub_page_id,
        )
        .await;

        let mut topic_data = topic.get_access().await;
        crate::operations::delivery::start_new(&app, &topic, &mut topic_data);
    });
}
