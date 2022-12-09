use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use std::sync::Arc;

use crate::app::AppContext;

use super::DeleteSessionInputContract;

#[my_http_server_swagger::http_route(
    method: "DELETE",
    route: "/Sessions",
    summary: "Kicks a session",
    description: "Kicks a session",
    input_data: "DeleteSessionInputContract",
    controller: "Sessions",
    result:[
        {status_code: 204, description: "Session is kicked"},    
    ]
)]
pub struct DeleteSessionAction {
    app: Arc<AppContext>,
}

impl DeleteSessionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DeleteSessionAction,
    input_data: DeleteSessionInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    match action.app.sessions.get(input_data.connection_id).await {
        Some(session) => {
            session.disconnect().await;
            HttpOutput::Empty.into_ok_result(true).into()
        }
        None => Err(HttpFailResult::as_not_found(
            format!("Session {} is not found", input_data.connection_id),
            false,
        )),
    }
}
