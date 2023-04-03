use my_http_server_swagger::MyHttpInput;

#[derive(Debug, MyHttpInput)]
pub struct EnableDebugInputModel {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name = "queueId"; description = "Id of queue")]
    pub queue_id: String,
}

#[derive(Debug, MyHttpInput)]
pub struct GetOnDeliveryInputModel {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name = "queueId"; description = "Id of queue")]
    pub queue_id: String,
    #[http_query(name = "subscriberId"; description = "Id of subscriber")]
    pub subscriber_id: i64,
}
