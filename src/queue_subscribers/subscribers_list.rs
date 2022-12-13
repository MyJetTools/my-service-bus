use std::{collections::HashMap, sync::Arc, time::Duration};

use my_service_bus_abstractions::{subscriber::TopicQueueType, MessageId};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    sessions::{MyServiceBusSession, SessionId},
    utils::MinMessageIdCalculator,
};

use super::{QueueSubscriber, SubscriberId};

pub enum SubscribersData {
    MultiSubscribers(HashMap<SubscriberId, QueueSubscriber>),
    SingleSubscriber(Option<QueueSubscriber>),
}

pub struct DeadSubscriber {
    pub subscriber_id: SubscriberId,
    pub session: Arc<MyServiceBusSession>,
    pub duration: Duration,
}

impl DeadSubscriber {
    pub fn new(subscriber: &QueueSubscriber, duration: Duration) -> Self {
        Self {
            session: subscriber.session.clone(),
            subscriber_id: subscriber.id,
            duration,
        }
    }
}

pub struct SubscribersList {
    data: SubscribersData,
    pub snapshot_id: usize,
    pub last_unsubscribe: DateTimeAsMicroseconds,
}

impl SubscribersList {
    pub fn new(queue_type: TopicQueueType) -> Self {
        let last_unsubscribe = DateTimeAsMicroseconds::now();
        match queue_type {
            TopicQueueType::Permanent => Self {
                snapshot_id: 0,
                data: SubscribersData::MultiSubscribers(HashMap::new()),
                last_unsubscribe,
            },
            TopicQueueType::DeleteOnDisconnect => Self {
                snapshot_id: 0,
                data: SubscribersData::MultiSubscribers(HashMap::new()),
                last_unsubscribe,
            },
            TopicQueueType::PermanentWithSingleConnection => Self {
                snapshot_id: 0,
                data: SubscribersData::SingleSubscriber(None),
                last_unsubscribe,
            },
        }
    }

    pub fn get_on_delivery_amount(&self) -> i64 {
        match &self.data {
            SubscribersData::MultiSubscribers(subscribers) => {
                let mut result = 0;
                for subscriber in subscribers.values() {
                    result += subscriber.get_on_delivery_amount();
                }

                result
            }
            SubscribersData::SingleSubscriber(subscriber) => {
                if let Some(subscriber) = subscriber {
                    subscriber.get_on_delivery_amount()
                } else {
                    0
                }
            }
        }
    }

    pub fn get_all(&self) -> Option<Vec<&QueueSubscriber>> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                if hash_map.len() == 0 {
                    return None;
                }

                return Some(hash_map.values().collect());
            }
            SubscribersData::SingleSubscriber(single) => {
                let subscriber = single.as_ref()?;
                Some(vec![subscriber])
            }
        }
    }

    pub fn get_min_message_id(&self) -> Option<MessageId> {
        match &self.data {
            SubscribersData::MultiSubscribers(subscribers) => {
                let mut min_message_id_calculator = MinMessageIdCalculator::new();

                for subscriber in subscribers.values() {
                    min_message_id_calculator.add(subscriber.get_min_message_id());
                }

                return min_message_id_calculator.value;
            }
            SubscribersData::SingleSubscriber(subscriber) => {
                let subscriber = subscriber.as_ref()?;
                return subscriber.get_min_message_id();
            }
        }
    }

    pub fn get_next_subscriber_ready_to_deliver(&self) -> Option<SubscriberId> {
        match &self.data {
            SubscribersData::MultiSubscribers(state) => {
                for subscriber in state.values() {
                    if subscriber.on_delivery.is_none() {
                        return Some(subscriber.id);
                    }
                }
            }
            SubscribersData::SingleSubscriber(state) => {
                if let Some(subscriber) = state {
                    if subscriber.on_delivery.is_none() {
                        return Some(subscriber.id);
                    }
                }
            }
        }

        None
    }

    pub fn get_by_id(&self, subscriber_id: SubscriberId) -> Option<&QueueSubscriber> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => return hash_map.get(&subscriber_id),
            SubscribersData::SingleSubscriber(single) => {
                if let Some(subscriber) = single {
                    if subscriber.id == subscriber_id {
                        return Some(subscriber);
                    }
                }

                return None;
            }
        }
    }

    pub fn get_by_id_mut(&mut self, subscriber_id: SubscriberId) -> Option<&mut QueueSubscriber> {
        match &mut self.data {
            SubscribersData::MultiSubscribers(hash_map) => return hash_map.get_mut(&subscriber_id),
            SubscribersData::SingleSubscriber(single) => {
                if let Some(subscriber) = single {
                    if subscriber.id == subscriber_id {
                        return Some(subscriber);
                    }
                }

                return None;
            }
        }
    }

    fn check_that_we_has_already_subscriber_for_that_session(&self, session_id: SessionId) -> bool {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                for subscriber in hash_map.values() {
                    if subscriber.session.id == session_id {
                        return false;
                    }
                }
            }
            SubscribersData::SingleSubscriber(single_subscriber) => {
                if let Some(subscriber) = single_subscriber {
                    if subscriber.session.id == session_id {
                        return false;
                    }
                }
            }
        }

        true
    }

    ///Returns the subscriber which is kicked
    pub fn subscribe(
        &mut self,
        subscriber_id: SubscriberId,
        topic_id: String,
        queue_id: String,
        session: Arc<MyServiceBusSession>,
    ) -> Option<QueueSubscriber> {
        if !self.check_that_we_has_already_subscriber_for_that_session(session.id) {
            panic!(
                "Somehow we subscribe second time to the same queue {}/{} the same session_id {} for the new subscriber. Most probably there is a bug on the client",
                topic_id, queue_id, subscriber_id
            );
        }
        self.snapshot_id += 1;

        match &mut self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                if hash_map.contains_key(&subscriber_id) {
                    panic!(
                        "Somehow we generated the same ID {} for the new subscriber {}/{}",
                        subscriber_id, topic_id, queue_id
                    );
                }

                let subscriber = QueueSubscriber::new(subscriber_id, topic_id, queue_id, session);

                hash_map.insert(subscriber_id, subscriber);

                return None;
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(subscriber) = single {
                    if subscriber.id == subscriber_id {
                        panic!(
                            "Somehow we generated the same ID {} for the new subscriber {}/{}",
                            subscriber_id, topic_id, queue_id
                        );
                    }
                }

                let mut old_subscriber = Some(QueueSubscriber::new(
                    subscriber_id,
                    topic_id,
                    queue_id,
                    session,
                ));

                std::mem::swap(&mut old_subscriber, single);

                return old_subscriber;
            }
        }
    }

    pub fn get_amount(&self) -> usize {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => hash_map.len(),
            SubscribersData::SingleSubscriber(single) => {
                if single.is_none() {
                    0
                } else {
                    1
                }
            }
        }
    }

    pub fn one_second_tick(&mut self) {
        match &mut self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                for queue_subscriber in hash_map.values_mut() {
                    queue_subscriber.metrics.one_second_tick()
                }
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(subscriber) = single {
                    subscriber.metrics.one_second_tick();
                }
            }
        }
    }

    fn resolve_subscriber_id_by_session_id(&self, session_id: SessionId) -> Option<SubscriberId> {
        match &self.data {
            SubscribersData::MultiSubscribers(hash_map) => {
                for sub in hash_map.values() {
                    if sub.session.id == session_id {
                        return Some(sub.id);
                    }
                }
            }
            SubscribersData::SingleSubscriber(single) => {
                if let Some(sub) = single {
                    if sub.session.id == session_id {
                        return Some(sub.id);
                    }
                }
            }
        }

        None
    }

    pub fn remove(&mut self, subscriber_id: SubscriberId) -> Option<QueueSubscriber> {
        match &mut self.data {
            SubscribersData::MultiSubscribers(multi) => {
                let result = multi.remove(&subscriber_id);
                if result.is_some() {
                    self.last_unsubscribe = DateTimeAsMicroseconds::now();
                }
                self.snapshot_id += 1;
                result
            }
            SubscribersData::SingleSubscriber(single) => {
                let mut result = None;

                if let Some(sub) = single {
                    if sub.id == subscriber_id {
                        self.last_unsubscribe = DateTimeAsMicroseconds::now();
                        std::mem::swap(&mut result, single);
                    }
                }
                self.snapshot_id += 1;
                result
            }
        }
    }

    pub fn remove_by_session_id(&mut self, session_id: SessionId) -> Option<QueueSubscriber> {
        let subscriber_id = self.resolve_subscriber_id_by_session_id(session_id)?;
        self.remove(subscriber_id)
    }

    pub fn find_subscribers_dead_on_delivery(
        &self,
        max_delivery_duration: Duration,
    ) -> Option<Vec<DeadSubscriber>> {
        match &self.data {
            SubscribersData::MultiSubscribers(subscribers) => {
                let mut result = None;

                for subscriber in subscribers.values() {
                    if let Some(duration) = subscriber.is_dead_on_delivery(max_delivery_duration) {
                        if result.is_none() {
                            result = Some(Vec::new());
                        }

                        if let Some(result) = &mut result {
                            result.push(DeadSubscriber::new(subscriber, duration));
                        }
                    }
                }

                return result;
            }
            SubscribersData::SingleSubscriber(state) => match state {
                Some(subscriber) => {
                    if let Some(duration) = subscriber.is_dead_on_delivery(max_delivery_duration) {
                        return Some(vec![DeadSubscriber::new(subscriber, duration)]);
                    }

                    return None;
                }
                None => return None,
            },
        }
    }
}
