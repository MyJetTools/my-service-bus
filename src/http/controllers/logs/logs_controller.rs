use my_http_server_swagger::http_route;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use rust_extensions::StopWatch;

use crate::app::AppContext;

#[http_route(method: "GET", route: "/Logs")]
pub struct GetLogsAction {
    app: Arc<AppContext>,
}

impl GetLogsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    _action: &GetLogsAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut sw = StopWatch::new();
    sw.start();
    let logs = crate::LOGS.get().await;

    return super::renderers::compile_result("logs", logs, sw);
}
