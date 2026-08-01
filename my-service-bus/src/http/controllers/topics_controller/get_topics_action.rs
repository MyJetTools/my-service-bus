use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::JsonTopicResult;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/api/Topics/Create",
    description: "Returns list of topics",
    summary: "Get list of topics",
    controller: "Topics",
    result:[
        {status_code: 200, description: "List of topics", model:"Vec<JsonTopicResult>"},
    ]
)]
pub struct GetTopicsAction {
    app: Arc<AppContext>,
}

impl GetTopicsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetTopicsAction,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let topics = namespace.topic_list.get_all();

    let mut items: Vec<JsonTopicResult> = Vec::new();

    for topic in topics.iter() {
        let item = JsonTopicResult::new(topic.as_ref()).await;

        items.push(item);
    }

    HttpOutput::as_json(items).into_ok_result(true).into()
}
