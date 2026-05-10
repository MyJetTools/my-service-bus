use my_service_bus::abstractions::MessageId;

#[derive(Debug)]
pub struct NextMessage {
    pub message_id: MessageId,
    pub attempt_no: i32,
}
