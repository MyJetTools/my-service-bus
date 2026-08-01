use std::sync::Arc;

use my_http_server::macros::http_route;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::{GreetingInputModel, GreetingJsonResult};

#[http_route(
    method: "POST",
    route: "/api/Greeting",
    deprecated_routes: ["/Greeting"],
    controller: "Greeting",
    description: "Issue new Http Session",
    summary: "Issues new Http Session",
    input_data: "GreetingInputModel",
    result: [
        {status_code: 200, description: "Session description", model: "GreetingJsonResult"},
        {status_code: 400, description: "Bad request"}, 
        {status_code: 401, description: "Unauthorized"},
    ]
)]
pub struct GreetingAction {
    app: Arc<AppContext>,
}

impl GreetingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GreetingAction,
    input_data: GreetingInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let ip = ctx.request.get_ip().get_real_ip().to_string();

    // The namespace of an HTTP client is fixed here and read from the session on
    // every later publish/subscribe, so a request that forgets the header can not
    // silently land in the default namespace.
    let namespace = crate::http::get_request_namespace(&action.app, ctx)?;

    let session_key =
        action
            .app
            .sessions
            .add_http(input_data.name, input_data.version, ip, namespace);

    let result = GreetingJsonResult {
        session: session_key.into_string(),
    };

    HttpOutput::as_json(result).into_ok_result(true).into()
}
