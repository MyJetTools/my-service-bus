use super::models::*;
use crate::app::AppContext;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/api/Debug/Console",
    input_data: GetDebugConsoleInputModel,
    description: "Returns the debug console records (date + data). Holds up to 1000 entries.",
    summary: "Get debug console records",
    controller: "Debug",
    result:[
        {status_code: 200, description: "Debug console content", model: DebugConsoleHttpModel},
    ]
)]
pub struct GetDebugConsoleAction {
    app: Arc<AppContext>,
}

impl GetDebugConsoleAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetDebugConsoleAction,
    input_data: GetDebugConsoleInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let tail = input_data
        .tail
        .and_then(|v| if v > 0 { Some(v as usize) } else { None });

    let records = action.app.debug_console.get_records(tail);
    let target = action.app.debug_console.get_target();

    if let Some(true) = input_data.clear {
        action.app.debug_console.clear();
    }

    let response = DebugConsoleHttpModel {
        enabled: target.is_some(),
        topic_id: target.as_ref().map(|t| t.topic_id.clone()),
        queue_id: target.as_ref().and_then(|t| t.queue_id.clone()),
        records_count: records.len(),
        records: records
            .into_iter()
            .map(|r| DebugConsoleRecordHttpModel {
                date: r.date_rfc3339(),
                data: r.message,
            })
            .collect(),
    };

    HttpOutput::as_json(response).into_ok_result(true).into()
}
