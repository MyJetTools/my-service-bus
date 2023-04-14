use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use rust_extensions::StopWatch;
use std::sync::Arc;

use crate::app::AppContext;

use super::models::ReadLogsByTopicInputModel;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Logs/Topic/{topicId}",
    input_data: "ReadLogsByTopicInputModel",
)]
pub struct GetLogsByTopicAction {
    app: Arc<AppContext>,
}

impl GetLogsByTopicAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetLogsByTopicAction,
    input_data: ReadLogsByTopicInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut sw = StopWatch::new();
    sw.start();
    let logs_result = action
        .app
        .logs
        .get_by_topic(input_data.topic_id.as_str())
        .await;

    match logs_result {
        Some(logs) => super::renderers::compile_result("logs by topic", logs, sw),
        None => {
            sw.pause();

            let content = format!(
                "Result compiled in: {:?}. No log records for the topic '{}'",
                sw.duration(),
                input_data.topic_id.as_str()
            );

            HttpOutput::Content {
                content_type: Some(WebContentType::Text),
                content: content.into_bytes(),
                headers: None,
            }
            .into_ok_result(false)
            .into()
        }
    }
}
