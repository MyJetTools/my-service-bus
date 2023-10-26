use super::models::*;
use crate::app::AppContext;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/Debug/GetQueues",
    input_data: "GetQueuesAwaitingToDeliver",
    description: "Get queues  to deliver",
    summary: "Returns queues awaiting to deliver",
    controller: "Debug",
    result:[
        {status_code: 202, description: "Debug mode is enabled", model: "Vec<QueueDebugModel>"},
    ]
)]
pub struct GetQueuesAwaitingToDeliverAction {
    app: Arc<AppContext>,
}

impl GetQueuesAwaitingToDeliverAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetQueuesAwaitingToDeliverAction,
    input_data: GetQueuesAwaitingToDeliver,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let topic = action.app.topic_list.get(&input_data.topic_id).await;

    if topic.is_none() {
        return Err(HttpFailResult::as_not_found(
            "Topic not found".to_string(),
            false,
        ));
    }

    let topic = topic.unwrap();

    let mut result = Vec::new();
    {
        let topic_data = topic.get_access().await;

        for queue in topic_data.queues.get_queues() {
            let mut subscribers = Vec::new();

            if let Some(the_subscribers) = queue.subscribers.get_all() {
                for subscriber in the_subscribers {
                    subscribers.push(QueueSubscriberDebugModel {
                        id: subscriber.id.get_value(),
                        session_id: subscriber.session.id.get_value(),
                        subscribed: subscriber.subscribed.to_rfc3339(),
                        delivery_status: format!("{:?}", subscriber.delivery_state),
                    });
                }
            }

            result.push(QueueDebugModel {
                name: queue.queue_id.to_string(),
                queue_type: format!("{:?}", queue.queue_type),
                subscribers,
            })
        }
    }

    HttpOutput::as_json(result).into_ok_result(false).into()
}
