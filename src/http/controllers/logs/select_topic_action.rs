use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use rust_extensions::StringBuilder;
use std::sync::Arc;

use crate::app::AppContext;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/api/Logs/Topic",
)]
pub struct SelectTopicAction {
    app: Arc<AppContext>,
}

impl SelectTopicAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SelectTopicAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line("<h1>Please, select topic to show logs</h1>");

    for topic in action.app.topic_list.get_all().await {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/api/logs/topic/{topic_id}'>{topic_id}</a>",
            topic_id = topic.topic_id
        );
        sb.append_line(line.as_str())
    }

    crate::http::html::compile("Select topic to show logs".to_string(), sb.to_string_utf8())
}
