use my_http_server::macros::MyHttpInput;

#[derive(MyHttpInput)]
pub struct GetListOfQueuesInputContract {
    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: String,
}

#[derive(MyHttpInput)]
pub struct DeleteQueueInputContract {
    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name="queueId"; description = "Id of queue")]
    pub queue_id: String,
}

#[derive(MyHttpInput)]
pub struct SetQueueMessageIdInputContract {
    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name="queueId"; description = "Id of queue")]
    pub queue_id: String,
    #[http_query(name="messageId"; description = "Message id")]
    pub message_id: i64,
}
