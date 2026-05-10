use dioxus_utils::DataState;

use crate::models::MySbHttpContract;

#[derive(Default)]
pub struct MySbState {
    pub started: bool,
    pub data: DataState<MySbHttpContract>,
    pub filter_string: String,
}
