use my_http_server::{HttpContext, HttpFailResult, HttpOkResult};
use rust_extensions::StringBuilder;

use crate::app::logs::SystemProcess;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/Logs/Process",
)]

pub struct SelectProcessAction;

impl SelectProcessAction {
    pub fn new() -> Self {
        Self
    }
}

async fn handle_request(
    _action: &SelectProcessAction,

    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut sb = StringBuilder::new();

    sb.append_line("<h1>Please, select process to show logs</h1>");

    for process in &SystemProcess::iterate() {
        let line = format!(
            "<a class='btn btn-sm btn-outline-primary' href='/logs/process/{process:?}'>{process:?}</a>",
            process = process
        );
        sb.append_line(line.as_str())
    }

    crate::http::html::compile("Select topic to show logs".to_string(), sb.to_string_utf8())
}
