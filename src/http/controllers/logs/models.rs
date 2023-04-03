use my_http_server_swagger::MyHttpInput;

#[derive(MyHttpInput)]
pub struct ReadLogsByProcessInputModel {
    #[http_path(name = "processId"; description = "Id of Process")]
    pub process_id: String,
}

#[derive(MyHttpInput)]
pub struct ReadLogsByTopicInputModel {
    #[http_path(name = "topicId"; description = "Id of Topic")]
    pub topic_id: String,
}
