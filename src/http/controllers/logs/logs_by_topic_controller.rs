use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};

use my_http_server_swagger::http_route;
use rust_extensions::{StopWatch, StringBuilder};
use std::sync::Arc;

use crate::app::AppContext;

use super::models::ReadLogsByTopicInputModel;

#[http_route(method: "GET", route: "/Logs/Topic/{topicId}", input_data: "ReadLogsByTopicInputModel")]
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
    if input_data.topic_id.is_none() {
        return render_select_topic(action.app.as_ref()).await;
    }

    let topic_id = input_data.topic_id.unwrap();

    let mut sw = StopWatch::new();
    sw.start();
    let logs_result = action.app.logs.get_by_topic(topic_id.as_str()).await;

    match logs_result {
        Some(logs) => super::renderers::compile_result("logs by topic", logs, sw),
        None => {
            sw.pause();

            let content = format!(
                "Result compiled in: {:?}. No log recods for the topic '{}'",
                sw.duration(),
                topic_id.as_str()
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

async fn render_select_topic(app: &AppContext) -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line("<h1>Please, select topic to show logs</h1>");

    for topic in app.topic_list.get_all().await {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/topic/{topic_id}'>{topic_id}</a>",
            topic_id = topic.topic_id
        );
        sb.append_line(line.as_str())
    }

    crate::http::html::compile(
        "Select topic to show logs".to_string(),
        sb.to_string_utf8().unwrap(),
    )
}
