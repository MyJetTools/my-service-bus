use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use my_http_server::macros::http_route;

use std::sync::Arc;

use super::*;

use crate::app::AppContext;

#[http_route(
    method: "GET",
    route: "/api/Queues",
    controller: "Queues",
    description: "Get list of queues",
    summary: "Returns list of queues",
    input_data: "GetListOfQueuesInputContract",
    result: [
        {status_code: 200, description: "Session description", model: "Vec<String>"},
        {status_code: 400, description: "Bad request"}, 
        {status_code: 401, description: "Unauthorized"},
    ]
)]
pub struct GetQueuesAction {
    app: Arc<AppContext>,
}

impl GetQueuesAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetQueuesAction,
    input_data: GetListOfQueuesInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let topic = action
        .app
        .topic_list
        .get(input_data.topic_id.as_str())
        .await;

    if topic.is_none() {
        return Err(HttpFailResult::as_not_found(
            format!("Topic {} not found", input_data.topic_id),
            false,
        ));
    }

    let topic = topic.unwrap();

    let mut result = Vec::new();

    {
        let topic_data = topic.get_access().await;
        for queue in topic_data.queues.get_all() {
            result.push(queue.queue_id.as_str().to_string());
        }
    }

    HttpOutput::as_json(result).into_ok_result(true).into()
}

/*
#[async_trait]
impl GetAction for QueuesController {
    fn get_route(&self) -> &str {
        "/Queues"
    }

    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            controller_name: "Queues",
            description: "Get list of queues",

            input_params: GetListOfQueuesInputContract::get_input_params().into(),
            results: vec![
                HttpResult {
                    http_code: 200,
                    description: "List of queues".to_string(),
                    nullable: false,
                    data_type: HttpDataType::ArrayOf(ArrayElement::SimpleType(
                        HttpSimpleType::String,
                    )),
                },
                response::topic_not_found(),
            ],
        }
        .into()
    }
}
*/
