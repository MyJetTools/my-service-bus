use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use super::models::*;
use crate::app::AppContext;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Topics",
    summary: "Create a new topic",
    description: "Creates new topic",
    input_data: "CreateTopicRequestContract",
    controller: "Topics",
    result:[
        {status_code: 204, description: "Topic is created"},    
    ]
)]
pub struct CreateTopicsAction {
    app: Arc<AppContext>,
}

impl CreateTopicsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &CreateTopicsAction,
    input_data: CreateTopicRequestContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::operations::publisher::create_topic_if_not_exists(
        &action.app,
        None,
        input_data.topic_id.as_ref(),
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
