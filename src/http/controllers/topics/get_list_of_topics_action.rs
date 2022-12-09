use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::*;

#[my_http_server_swagger::http_route(
    method: "GET",
    route: "/Topics",
    summary: "Get list of topics",
    description: "Returns list of topics",
    controller: "Topics",
    result:[
        {status_code: 200, description: "List of topics", model: "JsonTopicsResult"},    
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

pub async fn handle_request(
    action: &GetTopicsAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let topics = action.app.topic_list.get_all().await;

    let mut items: Vec<JsonTopicResult> = Vec::new();

    for topic in topics {
        let item = JsonTopicResult::new(topic.as_ref()).await;

        items.push(item);
    }

    let contract = JsonTopicsResult { items };

    HttpOutput::as_json(contract).into_ok_result(true).into()
}
