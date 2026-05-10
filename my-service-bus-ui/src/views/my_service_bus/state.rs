use dioxus_utils::DataState;

use crate::models::MySbHttpContract;

#[derive(Default)]
pub struct MySbState {
    pub started: bool,
    pub data: DataState<MySbHttpContract>,
    pub filter_string: String,
    pub dialog: Option<DialogState>,
}

#[derive(Clone)]
pub enum DialogState {
    DeleteTopic { topic_id: String },
    DeleteQueue { topic_id: String, queue_id: String },
}
