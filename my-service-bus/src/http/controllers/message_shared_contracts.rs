use my_http_server::macros::MyHttpObjectStructure;
use serde::*;

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct MessageKeyValueJsonModel {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct MessageToDeliverHttpContract {
    pub id: i64,
    pub attempt_no: i32,
    pub headers: Vec<MessageKeyValueJsonModel>,
    pub content: String,
}
