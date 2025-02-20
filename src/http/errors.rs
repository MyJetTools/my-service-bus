use my_http_server::HttpFailResult;

use crate::operations::OperationFailResult;

impl From<OperationFailResult> for HttpFailResult {
    fn from(src: OperationFailResult) -> Self {
        Self::as_forbidden(Some(format!("{:?}", src)))
    }
}
