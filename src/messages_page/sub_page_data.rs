use std::collections::HashMap;

use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus_shared::{protobuf_models::MessageProtobufModel, sub_page::SubPage};

use super::MessagesToPersistBucket;

pub struct SubPageData {
    pub sub_page: SubPage,
    pub messages_to_persist: QueueWithIntervals,
    pub persist_id: usize,
    on_persistence: HashMap<usize, QueueWithIntervals>,
}

impl SubPageData {
    pub fn new(sub_page: SubPage) -> Self {
        Self {
            sub_page,
            messages_to_persist: QueueWithIntervals::new(),
            persist_id: 0,
            on_persistence: HashMap::new(),
        }
    }

    pub fn compile_messages_to_persist(&mut self, topic_id: &str) -> MessagesToPersistBucket {
        let mut messages_to_persist = Vec::new();
        let mut ids = QueueWithIntervals::new();

        while let Some(message_id) = self.messages_to_persist.dequeue() {
            match self.sub_page.get_message(message_id.into()) {
                my_service_bus_shared::sub_page::GetMessageResult::Message(msg) => {
                    let model: MessageProtobufModel = msg.into();
                    messages_to_persist.push(model);
                    ids.enqueue(message_id);
                }
                my_service_bus_shared::sub_page::GetMessageResult::Missing => {
                    println!(
                        "Topic:{}. Somehow we can not find message {} to persist. It's missing",
                        topic_id, message_id
                    );
                }
                my_service_bus_shared::sub_page::GetMessageResult::GarbageCollected => {
                    println!(
                        "Topic:{}. Somehow we can not find message {} to persist. It's Garbage collected",
                        topic_id, message_id
                    );
                }
            }
        }

        let persist_id = self.persist_id;
        self.persist_id += 1;

        self.on_persistence.insert(persist_id, ids);
        MessagesToPersistBucket::new(persist_id, messages_to_persist)
    }

    pub fn commit_persisted_messages(
        &mut self,
        messages_ot_persist: &MessagesToPersistBucket,
        persisted: bool,
    ) {
        if let Some(ids) = self.on_persistence.remove(&messages_ot_persist.id) {
            for id in &ids {
                if !persisted {
                    self.messages_to_persist.enqueue(id);
                }
            }
        }
    }

    pub fn can_be_gced(&self) -> bool {
        self.messages_to_persist.len() == 0 && self.on_persistence.len() == 0
    }
}
