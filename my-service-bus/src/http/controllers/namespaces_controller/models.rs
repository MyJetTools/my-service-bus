use my_http_server::macros::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct NamespaceContract {
    pub name: String,
    #[serde(rename = "topicsAmount")]
    pub topics_amount: usize,
}
