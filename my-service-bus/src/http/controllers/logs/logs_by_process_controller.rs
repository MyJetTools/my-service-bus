use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use rust_extensions::StopWatch;

use crate::app::{logs::SystemProcess, AppContext};

use super::models::*;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/api/Logs/Process/{processId}",
    input_data: "ReadLogsByProcessInputModel"
)]

pub struct GetLogsByProcessAction {
    app: Arc<AppContext>,
}

impl GetLogsByProcessAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetLogsByProcessAction,
    input_data: ReadLogsByProcessInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let process = SystemProcess::parse(input_data.process_id.as_str());

    if process.is_none() {
        return HttpOutput::Content {
            content_type: Some(WebContentType::Text),
            content: format!("Invalid process name: {}", input_data.process_id).into(),
            headers: None,
        }
        .into_ok_result(false)
        .into();
    }

    let process = process.unwrap();

    let mut sw = StopWatch::new();
    sw.start();
    let logs_result = action.app.logs.get_by_process(process).await;

    match logs_result {
        Some(logs) => super::renderers::compile_result("logs by process", logs, sw),
        None => {
            sw.pause();

            HttpOutput::Content {
                content_type: Some(WebContentType::Text),
                content: format!(
                    "Result compiled in: {:?}. No log records for the process '{}'",
                    sw.duration(),
                    input_data.process_id
                )
                .into_bytes(),
                headers: None,
            }
            .into_ok_result(false)
            .into()
        }
    }
}
