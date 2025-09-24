use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use my_http_server::macros::http_route;

use crate::app::AppContext;

#[http_route(method: "GET", route: "/metrics")]
pub struct MetricsAction {
    app: Arc<AppContext>,
}

impl MetricsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &MetricsAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = action.app.prometheus.build();

    HttpOutput::Content {
        status_code: 200,
        headers: None,
        content_type: None,
        content: result,
        set_cookies: None,
    }
    .into_ok_result(true)
}
