use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::*;

#[my_http_server_swagger::http_route(
    method: "DELETE",
    route: "/Topics/Delete",
    description: "Soft deletes topic",
    summary: "Delete topic",
    input_data: "DeleteTopicRequestContract",
    controller: "Topics",
    result:[
        {status_code: 202, description: "Topic is soft deleted"},
    ]
)]
pub struct DeleteTopicAction {
    app: Arc<AppContext>,
}

impl DeleteTopicAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DeleteTopicAction,
    input_data: DeleteTopicRequestContract,
    _: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::operations::delete_topic(
        &action.app,
        &input_data.topic_id,
        input_data.hard_delete_moment,
    )
    .await?;
    HttpOutput::Empty.into_ok_result(true).into()
}
