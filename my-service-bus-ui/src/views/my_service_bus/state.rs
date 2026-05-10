use dioxus_utils::DataState;

use crate::models::MySbHttpContract;

#[derive(Default)]
pub struct MySbState {
    pub started: bool,
    pub data: DataState<MySbHttpContract>,
    pub filter_string: String,
    /// (topic_id, queue_id) of the queue being confirmed for deletion.
    pub delete_queue_dialog: Option<(String, String)>,
    /// topic_id of the topic being confirmed for deletion.
    pub delete_topic_dialog: Option<String>,
}
