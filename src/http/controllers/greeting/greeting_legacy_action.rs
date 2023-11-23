use std::sync::Arc;

use my_http_server::macros::http_route;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, sessions::HttpConnectionData};

use super::models::{GreetingInputModel, GreetingJsonResult};

#[http_route(
    method: "POST",
    route: "/Greeting",
    controller: "Greeting",
    description: "Issue new Http Session (Legacy. Please use /api/Greeting)",
    summary: "Issues new Http Session (Legacy. Please use /api/Greeting)",
    input_data: "GreetingInputModel",
    deprecated: true,
    result: [
        {status_code: 200, description: "Session description", model: "GreetingJsonResult"},
        {status_code: 400, description: "Bad request"}, 
        {status_code: 401, description: "Unauthorized"},
    ]
)]
pub struct GreetingLegacyAction {
    app: Arc<AppContext>,
}

impl GreetingLegacyAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GreetingLegacyAction,
    input_data: GreetingInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let ip = ctx.request.get_ip().get_real_ip().to_string();

    let id = uuid::Uuid::new_v4().to_string();

    let data = HttpConnectionData::new(id.to_string(), input_data.name, input_data.version, ip);

    action.app.sessions.add_http(data).await;

    let result = GreetingJsonResult { session: id };

    HttpOutput::as_json(result).into_ok_result(true).into()
}
