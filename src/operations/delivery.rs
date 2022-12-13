use std::sync::Arc;

use crate::{
    app::AppContext,
    topics::{Topic, TopicData},
};

pub async fn start_new(app: &Arc<AppContext>, topic: &Arc<Topic>, topic_data: &mut TopicData) {
    if let Some(to_load) = topic_data.try_to_deliver(app.get_max_delivery_size()).await {
        for sub_page_id in to_load {
            crate::operations::load_page_and_try_to_deliver_again(
                app.clone(),
                topic.clone(),
                sub_page_id,
            );
        }
    }
}
