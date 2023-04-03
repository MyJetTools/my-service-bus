use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::CreateTopicRequestContract;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Topics/Create",
    input_data: "CreateTopicRequestContract",
    description: "Create topic",
    summary: "Create topic",
    controller: "Topics",
    result:[
        {status_code: 202, description: "Topic is created"},
    ]
)]
pub struct CreateTopicAction {
    app: Arc<AppContext>,
}

impl CreateTopicAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &CreateTopicAction,
    input_data: CreateTopicRequestContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::operations::publisher::create_topic_if_not_exists(
        &action.app,
        None,
        input_data.topic_id.as_ref(),
    )
    .await?;

    HttpOutput::as_text("Topic is created".to_string())
        .into_ok_result(true)
        .into()
}
