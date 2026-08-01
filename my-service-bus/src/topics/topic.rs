use std::time::Duration;

use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::sub_page::SubPageId;

use parking_lot::Mutex;
use rust_extensions::sorted_vec::EntityWithStrKey;

use crate::messages_page::{MessagesToPersistBucket, MySbMessageContent, SizeMetrics};
use crate::queue_subscribers::DeadSubscriber;

use super::topic_data_access::TopicDataAccess;
use super::{TopicId, TopicInner};

pub struct Topic {
    /// Namespace this topic belongs to. Kept on the topic itself so that
    /// everything downstream — metrics, logs, persistence snapshots — can name the
    /// topic unambiguously: `topic_id` alone is unique only inside one namespace.
    pub namespace: String,
    pub topic_id: TopicId,
    inner: Mutex<TopicInner>,
}

impl Topic {
    pub fn new(
        namespace: String,
        topic_id: String,
        message_id: i64,
        persist: bool,
        deleted: i64,
    ) -> Self {
        let topic_id = TopicId::new(topic_id);
        Self {
            namespace,
            topic_id: topic_id.clone(),
            inner: Mutex::new(TopicInner::new(
                topic_id, message_id, persist, deleted,
            )),
        }
    }

    /// Namespace as the persistence contract wants it: `None` for the default one,
    /// which is what a node knowing nothing about namespaces sends.
    pub fn as_grpc_namespace(&self) -> Option<String> {
        if self.namespace == my_service_bus::shared::validators::DEFAULT_NAMESPACE {
            None
        } else {
            Some(self.namespace.clone())
        }
    }

    pub fn get_access<'s>(&'s self) -> TopicDataAccess<'s> {
        let access = self.inner.lock();
        TopicDataAccess::new(access)
    }

    pub fn get_message_id(&self) -> MessageId {
        let read_access = self.inner.lock();
        read_access.message_id.into()
    }

    pub fn get_topic_info<TResult>(&self, convert: impl Fn(&TopicInner) -> TResult) -> TResult {
        let inner = self.inner.lock();

        convert(&inner)
    }

    pub fn find_subscribers_dead_on_delivery(
        &self,
        delivery_timeout_duration: Duration,
    ) -> Vec<DeadSubscriber> {
        let mut result = vec![];
        let mut topic_data = self.inner.lock();

        for queue in topic_data.queues.get_all_mut() {
            let dead_subscribers = queue
                .subscribers
                .find_subscribers_dead_on_delivery(delivery_timeout_duration);
            if dead_subscribers.len() > 0 {
                for dead_subscriber in dead_subscribers {
                    if !result.iter().any(|itm: &DeadSubscriber| {
                        itm.subscriber_id.equals_to(dead_subscriber.subscriber_id)
                    }) {
                        result.push(dead_subscriber);
                    }
                }
            }
        }

        result
    }

    pub fn get_messages_to_persist<TResult>(
        &self,
        transform: impl Fn(&MySbMessageContent) -> TResult,
    ) -> Vec<(SubPageId, Vec<TResult>)> {
        let read_access = self.get_access();
        read_access.get_messages_to_persist(transform)
    }

    pub fn mark_messages_as_persisted(&self, bucket: &MessagesToPersistBucket) {
        let mut write_access = self.get_access();
        write_access.mark_messages_as_persisted(bucket.sub_page_id, &bucket.ids);
    }

    pub fn get_topic_size_metrics(&self) -> SizeMetrics {
        let read_access = self.get_access();
        read_access.get_topic_size_metrics()
    }

    pub fn update_persist(&self, persist: bool) {
        let mut write_access = self.get_access();
        write_access.persist = persist;
    }

    pub fn get_deleted(&self) -> i64 {
        self.inner.lock().deleted
    }

    pub fn set_deleted(&self, deleted: i64) {
        self.inner.lock().deleted = deleted;
    }
}

impl EntityWithStrKey for Topic {
    fn get_key(&self) -> &str {
        self.topic_id.as_str()
    }
}
