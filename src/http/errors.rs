use my_http_server::{HttpFailResult, WebContentType};

use crate::operations::OperationFailResult;

impl From<OperationFailResult> for HttpFailResult {
    fn from(src: OperationFailResult) -> Self {
        Self::as_forbidden(Some(format!("{:?}", src)))
    }
}

pub trait AsHttpFailResult {
    fn as_fail_result(self) -> HttpFailResult;
}

impl AsHttpFailResult for hyper::Error {
    fn as_fail_result(self) -> HttpFailResult {
        HttpFailResult {
            content_type: WebContentType::Text,
            status_code: 500,
            content: format!("Can not get body from Request. Err:{:?}", self).into_bytes(),
            write_telemetry: true,
            write_to_log: false,
        }
    }
}
