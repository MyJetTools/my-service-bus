use super::models::*;
use crate::app::AppContext;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

#[my_http_server::macros::http_route(
    method: "POST",
    route: "/api/Debug/Console/Target",
    input_data: SetDebugConsoleTargetInputModel,
    description: "Sets (or switches) which topic/queue the debug console records. To turn it off use DELETE /api/Debug/Console.",
    summary: "Set debug console target",
    controller: "Debug",
    result:[
        {status_code: 200, description: "Debug console target updated"},
    ]
)]
pub struct SetDebugConsoleTargetAction {
    app: Arc<AppContext>,
}

impl SetDebugConsoleTargetAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SetDebugConsoleTargetAction,
    input_data: SetDebugConsoleTargetInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let topic_id = input_data.topic_id.trim().to_string();
    if topic_id.is_empty() {
        return Err(HttpFailResult::as_validation_error("topicId is required"));
    }

    let queue_id = input_data
        .queue_id
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    action
        .app
        .debug_console
        .set_target(topic_id.clone(), queue_id.clone());

    let response = match queue_id {
        Some(queue_id) => format!(
            "Debug console ON. Tracing topic '{}', queue '{}'. Buffer cleared.",
            topic_id, queue_id
        ),
        None => format!(
            "Debug console ON. Tracing topic '{}', all queues. Buffer cleared.",
            topic_id
        ),
    };

    HttpOutput::as_text(response).into_ok_result(true).into()
}
