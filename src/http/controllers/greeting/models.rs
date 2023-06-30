use my_http_server_swagger::{MyHttpInput, MyHttpObjectStructure};
use serde::{Deserialize, Serialize};

#[derive(Debug, MyHttpInput)]
pub struct GreetingInputModel {
    #[http_query(description = "Name of application")]
    pub name: String,

    #[http_query(description = "Version of application")]
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct GreetingJsonResult {
    pub session: String,
}
