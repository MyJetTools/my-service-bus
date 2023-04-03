use crate::app::AppContext;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use std::sync::Arc;

#[my_http_server_swagger::http_route(
    method: "DELETE",
    route: "/Debug/Disable",
    description: "Disable debug mode",
    summary: "Disables debug mode",
    controller: "Debug",
    result:[
        {status_code: 202, description: "Debug mode is enabled"},
    ]
)]
pub struct DisableDebugModeAction {
    app: Arc<AppContext>,
}

impl DisableDebugModeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DisableDebugModeAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    action.app.disable_debug_topic_and_queue().await;

    HttpOutput::Empty.into_ok_result(true).into()
}
