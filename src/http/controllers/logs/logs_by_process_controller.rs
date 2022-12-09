use my_http_server_swagger::http_route;
use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput, WebContentType};
use rust_extensions::{StopWatch, StringBuilder};

use crate::app::{logs::SystemProcess, AppContext};

use super::models::ReadLogsByProcessInputModel;

#[http_route(method: "GET", route: "/Logs/Process/{processId}", input_data: "ReadLogsByProcessInputModel")]
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
    if input_data.process_id.is_none() {
        return render_select_process().await;
    }

    let process_id = input_data.process_id.unwrap();

    let process = SystemProcess::parse(process_id.as_str());

    if process.is_none() {
        return HttpOutput::Content {
            content_type: Some(WebContentType::Text),
            content: format!("Invalid process name: {}", process_id).into(),
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
                    "Result compiled in: {:?}. No log recods for the process '{}'",
                    sw.duration(),
                    process_id
                )
                .into_bytes(),
                headers: None,
            }
            .into_ok_result(false)
            .into()
        }
    }
}

async fn render_select_process() -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line("<h1>Please, select process to show logs</h1>");

    for process in &SystemProcess::iterate() {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/process/{process:?}'>{process:?}</a>",
            process = process
        );
        sb.append_line(line.as_str())
    }

    crate::http::html::compile(
        "Select topic to show logs".to_string(),
        sb.to_string_utf8().unwrap(),
    )
}
