use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus::shared::{protobuf_models::MessageProtobufModel, sub_page::SubPageId};

pub struct MessagesToPersistBucket {
    messages_to_persist: Option<Vec<MessageProtobufModel>>,
    pub sub_page_id: SubPageId,
    pub ids: QueueWithIntervals,
    pub size: usize,
}

impl MessagesToPersistBucket {
    pub fn new(sub_page_id: SubPageId) -> Self {
        Self {
            messages_to_persist: Some(Vec::new()),
            sub_page_id,
            ids: QueueWithIntervals::new(),
            size: 0,
        }
    }

    pub fn add(&mut self, msg: MessageProtobufModel) {
        let msg_id = msg.get_message_id();
        self.size += msg.data.len();
        self.messages_to_persist.as_mut().unwrap().push(msg);
        self.ids.enqueue(msg_id.get_value());
    }

    pub fn get(&mut self) -> Vec<MessageProtobufModel> {
        self.messages_to_persist.take().unwrap()
    }
}
