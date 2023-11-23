use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use std::sync::Arc;

use crate::app::AppContext;

use super::models::GetOnDeliveryInputModel;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/api/Debug/OnDelivery",
    input_data: "GetOnDeliveryInputModel",
    description: "Show messages on delivery",
    summary: "Shows messages on delivery",
    controller: "Debug",
    result:[
        {status_code: 200, description: "Ids of subscribers on delivery"},
    ]
)]
pub struct OnDeliveryAction {
    app: Arc<AppContext>,
}

impl OnDeliveryAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &OnDeliveryAction,
    input_model: GetOnDeliveryInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let topic = action
        .app
        .topic_list
        .get(input_model.topic_id.as_str())
        .await;
    if topic.is_none() {
        return Err(HttpFailResult::as_not_found(
            "Topic not found".to_string(),
            false,
        ));
    }

    let topic = topic.unwrap();

    let ids = {
        let topic_data = topic.get_access().await;

        let queue = topic_data.queues.get(input_model.queue_id.as_str());

        if queue.is_none() {
            return Err(HttpFailResult::as_not_found(
                "Queue not found".to_string(),
                false,
            ));
        }

        let queue = queue.unwrap();

        queue.get_messages_on_delivery(input_model.subscriber_id.into())
    };

    HttpOutput::as_text(format!("{:?}", ids))
        .into_ok_result(true)
        .into()
}
