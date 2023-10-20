use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use rust_extensions::StopWatch;

use crate::app::AppContext;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/Logs",
)]
pub struct LogsAction {
    app: Arc<AppContext>,
}

impl LogsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &LogsAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut sw = StopWatch::new();
    sw.start();
    let logs = action.app.logs.get().await;

    return super::renderers::compile_result("logs", logs, sw);
}
