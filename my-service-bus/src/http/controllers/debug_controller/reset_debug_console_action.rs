use crate::app::AppContext;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

#[my_http_server::macros::http_route(
    method: "DELETE",
    route: "/api/Debug/Console",
    description: "Resets the debug console: turns tracing OFF (clears the active target) and empties the buffer.",
    summary: "Reset debug console (off + clear)",
    controller: "Debug",
    result:[
        {status_code: 200, description: "Debug console turned off and buffer cleared"},
    ]
)]
pub struct ResetDebugConsoleAction {
    app: Arc<AppContext>,
}

impl ResetDebugConsoleAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &ResetDebugConsoleAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    action.app.debug_console.disable();
    action.app.debug_console.clear();

    HttpOutput::as_text("Debug console reset: tracing OFF, buffer cleared.")
        .into_ok_result(true)
        .into()
}
