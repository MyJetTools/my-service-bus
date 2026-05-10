use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use my_http_server::macros::{http_route, MyHttpInput};

use std::sync::Arc;



use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/Queues/SetMaxMessagePerPayload",
    controller: "Queues",
    description: "Set max message per delivery payload",
    summary: "Set max message per delivery payload",
    input_data: SetMaxMessagesPerPayloadInputModel,
    result: [
        {status_code: 202, description: "Set max messages"},
   
    ]
)]
pub struct SetMaxMessagePerPayloadAction {
    app: Arc<AppContext>,
}

impl SetMaxMessagePerPayloadAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SetMaxMessagePerPayloadAction,
    input_data: SetMaxMessagesPerPayloadInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {

    let max_messages = match input_data.max_messages{
        Some(messages) => {
            if messages == 0{
                None
            }else {
                Some(messages)
            }
        },
        None => None,
    };


    crate::operations::queues::set_max_messages_per_payload(
        action.app.as_ref(),
        input_data.topic_id.as_str(),
        input_data.queue_id.as_str(),
        max_messages,
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}



#[derive(MyHttpInput)]
pub struct SetMaxMessagesPerPayloadInputModel {
    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name="queueId"; description = "Id of queue")]
    pub queue_id: String,
    #[http_query(name="maxMessages"; description = "Max messages - 0 - un")]
    pub max_messages: Option<usize>,
}
