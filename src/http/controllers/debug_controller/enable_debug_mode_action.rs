use super::models::EnableDebugInputModel;
use crate::app::AppContext;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

#[my_http_server_swagger::http_route(
    method: "POST",
    route: "/Debug/Enable",
    input_data: "EnableDebugInputModel",
    description: "Enable debug mode for specific queue",
    summary: "Enables debug mode for specific queue",
    controller: "Debug",
    result:[
        {status_code: 202, description: "Debug mode is enabled"},
    ]
)]
pub struct EnableDebugModeAction {
    app: Arc<AppContext>,
}

impl EnableDebugModeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &EnableDebugModeAction,
    input_data: EnableDebugInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    action
        .app
        .set_debug_topic_and_queue(input_data.topic_id.as_ref(), input_data.queue_id.as_ref())
        .await;

    HttpOutput::Empty.into_ok_result(true).into()
}
