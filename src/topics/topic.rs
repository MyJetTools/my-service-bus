use std::sync::atomic::AtomicBool;
use std::time::Duration;

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::page_id::PageId;
use my_service_bus_shared::sub_page::SubPageId;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::queue_subscribers::DeadSubscriber;

use super::topic_data::TopicData;
use super::topic_data_access::TopicDataAccess;
use super::TopicSnapshot;

pub struct Topic {
    pub topic_id: String,
    data: Mutex<TopicData>,
    pub restore_page_lock: Mutex<DateTimeAsMicroseconds>,
    pub immediately_persist_is_charged: AtomicBool,
}

impl Topic {
    pub fn new(topic_id: String, message_id: i64) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            data: Mutex::new(TopicData::new(topic_id, message_id)),
            restore_page_lock: Mutex::new(DateTimeAsMicroseconds::now()),
            immediately_persist_is_charged: AtomicBool::new(false),
        }
    }

    pub async fn get_access<'s>(&'s self, process: &'static str) -> TopicDataAccess<'s> {
        println!(
            "{}. Getting access with process: {}",
            self.topic_id, process
        );
        let access = self.data.lock().await;

        let process_taken = {
            let mut process_taken = access.process_taken.lock().await;
            process_taken.push(process);
            access.process_taken.clone()
        };

        println!("{}. Got access with process: {}", self.topic_id, process);
        TopicDataAccess::new(access, process_taken, process)
    }

    pub async fn get_message_id(&self) -> MessageId {
        let read_access = self.data.lock().await;
        read_access.message_id.into()
    }

    pub async fn get_current_page(&self) -> (PageId, SubPageId) {
        let read_access = self.data.lock().await;

        let page_id = PageId::from_message_id(read_access.message_id.into());
        let sub_page_id = SubPageId::from_message_id(read_access.message_id.into());

        (page_id, sub_page_id)
    }

    pub async fn get_topic_snapshot(&self) -> TopicSnapshot {
        let topic_data = self.data.lock().await;

        TopicSnapshot {
            message_id: topic_data.message_id.into(),
            topic_id: topic_data.topic_id.as_str().into(),
            queues: topic_data.queues.get_snapshot_to_persist(),
        }
    }

    pub async fn find_subscribers_dead_on_delivery(
        &self,
        delivery_timeout_duration: Duration,
    ) -> Option<Vec<DeadSubscriber>> {
        let mut result = None;
        let mut topic_data = self.data.lock().await;

        for queue in topic_data.queues.get_all_mut() {
            if let Some(dead_subscribers) = queue
                .subscribers
                .find_subscribers_dead_on_delivery(delivery_timeout_duration)
            {
                if result.is_none() {
                    result = Some(Vec::new());
                }

                let result_mut = result.as_mut().unwrap();

                for dead_subscriber in dead_subscribers {
                    if result_mut
                        .iter()
                        .position(|itm: &DeadSubscriber| {
                            itm.subscriber_id.equals_to(dead_subscriber.subscriber_id)
                        })
                        .is_none()
                    {
                        result_mut.push(dead_subscriber);
                    }
                }
            }
        }

        result
    }
}
