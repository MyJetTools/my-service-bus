use crate::http::auth::GetSessionToken;

use my_http_server::macros::http_route;
use my_service_bus::abstractions::publisher::MessageToPublish;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::contracts::PublishMessageHttpInput;

#[http_route(
    method: "POST",
    route: "/api/Publish",
    deprecated_routes: ["/Publish"],
    controller: "Publish",
    description: "Publish messages to topic",
    summary: "Publishes messages to topic",
    input_data: "PublishMessageHttpInput",
    authorized: "Yes",
    result: [
        {status_code: 202, description: "Message is published"},
    ]
)]
pub struct PublishAction {
    app: Arc<AppContext>,
}

impl PublishAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &PublishAction,
    http_input: PublishMessageHttpInput,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let session = ctx.get_http_session(&action.app).await?;

    let mut messages_to_publish = Vec::with_capacity(http_input.messages.len());

    let mut content_size = 0;

    for mut msg_in_json in http_input.messages {
        let msg = MessageToPublish {
            headers: msg_in_json.get_headers(),
            content: msg_in_json.get_content()?,
        };

        content_size += msg.content.len();

        messages_to_publish.push(msg);
    }

    crate::operations::publisher::publish(
        &action.app,
        http_input.topic_id.as_str(),
        messages_to_publish,
        false,
        session.id,
    )
    .await?;

    let http_session = session.connection.unwrap_as_http();

    http_session.update_written_amount(content_size);

    HttpOutput::Empty.into_ok_result(true).into()
}
