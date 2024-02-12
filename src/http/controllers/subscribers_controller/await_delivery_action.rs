use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, http::auth::GetSessionToken};

use super::contracts::*;

#[my_http_server::macros::http_route(
    method: "POST",
    route: "/api/Subscribers/Await",
    description: "Await new messages",
    summary: "Await new messages",
    controller: "Subscribers",
    result:[
        {status_code: 200, description: "Long pooling Delivery. Please use it in immediate loop", model:AwaitDeliveryHttpResponse},
    ]
)]
pub struct AwaitDeliveryAction {
    app: Arc<AppContext>,
}

impl AwaitDeliveryAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &AwaitDeliveryAction,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let session = ctx.get_http_session(&action.app).await?;

    session.connection.unwrap_as_http().ping();

    match session.get_long_pool_messages().await {
        Ok(messages) => match messages {
            None => HttpOutput::Empty.into_ok_result(false),
            Some(package) => {
                let response = AwaitDeliveryHttpResponse {
                    topic_id: package.topic_id.to_string(),
                    queue_id: package.queue_id.to_string(),

                    confirmation_id: package.subscriber_id.get_value(),
                    messages: package.messages,
                };

                HttpOutput::as_json(response).into_ok_result(false)
            }
        },
        Err(err) => {
            return Err(HttpFailResult::as_fatal_error(err));
        }
    }
}
