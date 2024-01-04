use std::sync::atomic::AtomicBool;
use std::time::Duration;

use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::sub_page::SubPageId;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::messages_page::{MessagesToPersistBucket, SizeMetrics};
use crate::queue_subscribers::DeadSubscriber;

use super::topic_data_access::TopicDataAccess;
use super::TopicInner;
use super::TopicSnapshot;

pub struct Topic {
    pub topic_id: String,
    inner: Mutex<TopicInner>,
    pub restore_page_lock: Mutex<DateTimeAsMicroseconds>,
    pub immediately_persist_is_charged: AtomicBool,
}

impl Topic {
    pub fn new(topic_id: String, message_id: i64, persist: bool) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            inner: Mutex::new(TopicInner::new(topic_id, message_id, persist)),
            restore_page_lock: Mutex::new(DateTimeAsMicroseconds::now()),
            immediately_persist_is_charged: AtomicBool::new(false),
        }
    }

    pub async fn get_access<'s>(&'s self) -> TopicDataAccess<'s> {
        let access = self.inner.lock().await;
        TopicDataAccess::new(access)
    }

    pub async fn get_message_id(&self) -> MessageId {
        let read_access = self.inner.lock().await;
        read_access.message_id.into()
    }

    pub async fn get_current_sub_page(&self) -> SubPageId {
        let read_access = self.inner.lock().await;

        let sub_page_id = SubPageId::from_message_id(read_access.message_id.into());

        sub_page_id
    }

    pub async fn get_topic_snapshot(&self) -> TopicSnapshot {
        let inner = self.inner.lock().await;

        TopicSnapshot {
            message_id: inner.message_id.into(),
            topic_id: inner.topic_id.as_str().into(),
            queues: inner.queues.get_snapshot_to_persist(),
            persist: inner.persist,
        }
    }

    pub async fn find_subscribers_dead_on_delivery(
        &self,
        delivery_timeout_duration: Duration,
    ) -> Option<Vec<DeadSubscriber>> {
        let mut result = None;
        let mut topic_data = self.inner.lock().await;

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

    pub async fn get_messages_to_persist(
        &self,
        max_payload_size: usize,
    ) -> Option<MessagesToPersistBucket> {
        let read_access = self.get_access().await;

        read_access.pages.get_messages_to_persist(max_payload_size)
    }

    pub async fn mark_messages_as_persisted(&self, bucket: &MessagesToPersistBucket) {
        let mut write_access = self.get_access().await;
        write_access.pages.mark_messages_as_persisted(bucket);
    }

    pub async fn get_topic_size_metrics(&self) -> SizeMetrics {
        let read_access = self.get_access().await;
        read_access.get_topic_size_metrics()
    }

    pub async fn update_persist(&self, persist: bool) {
        let mut write_access = self.get_access().await;
        write_access.persist = persist;
    }
}
