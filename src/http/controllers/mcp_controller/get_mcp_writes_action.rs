use std::sync::Arc;

use my_http_server::macros::http_route;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::McpWritesContract;

#[http_route(
    method: "GET",
    route: "/api/Mcp/Writes",
    controller: "Mcp",
    description: "Tells whether the MCP write tools are currently enabled and how long is left of the window",
    summary: "MCP writes state",
    result: [
        {status_code: 200, description: "State of the write window", model: "McpWritesContract"},
    ]
)]
pub struct GetMcpWritesAction {
    app: Arc<AppContext>,
}

impl GetMcpWritesAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetMcpWritesAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    HttpOutput::as_json(McpWritesContract::new(action.app.as_ref()))
        .into_ok_result(false)
        .into()
}
