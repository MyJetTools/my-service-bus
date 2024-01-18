use std::sync::atomic::AtomicBool;
use std::time::Duration;

use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::sub_page::SubPageId;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_extensions::sorted_vec::EntityWithStrKey;
use tokio::sync::Mutex;

use crate::messages_page::{MessagesToPersistBucket, MySbMessageContent, SizeMetrics};
use crate::queue_subscribers::DeadSubscriber;

use super::topic_data_access::TopicDataAccess;
use super::TopicSnapshot;
use super::{TopicId, TopicInner};

pub struct Topic {
    pub topic_id: TopicId,
    inner: Mutex<TopicInner>,
    pub restore_page_lock: Mutex<DateTimeAsMicroseconds>,
    pub immediately_persist_is_charged: AtomicBool,
}

impl Topic {
    pub fn new(topic_id: String, message_id: i64, persist: bool) -> Self {
        let topic_id = TopicId::new(topic_id);
        Self {
            topic_id: topic_id.clone(),
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

    pub async fn get_messages_to_persist<TResult>(
        &self,
        transform: impl Fn(&MySbMessageContent) -> TResult,
    ) -> Vec<(SubPageId, Vec<TResult>)> {
        let read_access = self.get_access().await;
        read_access.get_messages_to_persist(transform)
    }

    pub async fn mark_messages_as_persisted(&self, bucket: &MessagesToPersistBucket) {
        let mut write_access = self.get_access().await;
        write_access.mark_messages_as_persisted(bucket.sub_page_id, &bucket.ids);
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

impl EntityWithStrKey for Topic {
    fn get_key(&self) -> &str {
        self.topic_id.as_str()
    }
}
