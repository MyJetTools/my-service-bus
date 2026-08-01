use std::time::Duration;

use my_service_bus::abstractions::MessageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    queues::QueueId,
    sessions::{MyServiceBusSession, SessionId},
    topics::TopicId,
    utils::*,
};

use super::{QueueSubscriber, SubscriberId};

pub struct DeadSubscriber {
    pub subscriber_id: SubscriberId,
    pub session: MyServiceBusSession,
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
    subscribers: Vec<QueueSubscriber>,
    pub snapshot_id: usize,
    pub last_unsubscribe: DateTimeAsMicroseconds,
}

impl SubscribersList {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            snapshot_id: 0,
            last_unsubscribe: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn get_on_delivery_amount(&self) -> usize {
        let mut result = 0;
        for subscriber in self.subscribers.iter() {
            result += subscriber.get_on_delivery_amount();
        }
        result
    }

    pub fn get_all(&self) -> Option<Vec<&QueueSubscriber>> {
        if self.subscribers.is_empty() {
            return None;
        }
        Some(self.subscribers.iter().collect())
    }

    pub fn get_min_message_id(&self) -> Option<MessageId> {
        let mut min_message_id_calculator = MinMessageIdCalculator::new();

        for subscriber in self.subscribers.iter() {
            min_message_id_calculator.add(subscriber.get_min_message_id());
        }

        min_message_id_calculator.get()
    }

    pub fn get_and_rent_next_subscriber_ready_to_deliver(
        &mut self,
    ) -> Option<(SubscriberId, MyServiceBusSession)> {
        for subscriber in self.subscribers.iter_mut() {
            if subscriber.rent_me() {
                return Some((subscriber.id, subscriber.session.clone()));
            }
        }
        None
    }

    pub fn cancel_rent(&mut self, subscriber_id: SubscriberId) {
        if let Some(subscriber) = self
            .subscribers
            .iter_mut()
            .find(|s| s.id.equals_to(subscriber_id))
        {
            subscriber.cancel_the_rent();
        }
    }

    pub fn get_by_id(&self, subscriber_id: SubscriberId) -> Option<&QueueSubscriber> {
        self.subscribers
            .iter()
            .find(|s| s.id.equals_to(subscriber_id))
    }

    pub fn get_by_id_mut(&mut self, subscriber_id: SubscriberId) -> Option<&mut QueueSubscriber> {
        self.subscribers
            .iter_mut()
            .find(|s| s.id.equals_to(subscriber_id))
    }

    fn has_subscriber_for_session(&self, session_id: SessionId) -> bool {
        self.subscribers
            .iter()
            .any(|s| s.session.session_id == session_id)
    }

    ///Returns subscribers which were kicked (empty if none)
    pub fn subscribe(
        &mut self,
        single_connection: bool,
        subscriber_id: SubscriberId,
        topic_id: TopicId,
        queue_id: QueueId,
        session: MyServiceBusSession,
    ) -> Vec<QueueSubscriber> {
        if self.has_subscriber_for_session(session.session_id) {
            panic!(
                "Somehow we subscribe second time to the same queue {}/{} the same session_id {} for the new subscriber. Most probably there is a bug on the client",
                topic_id.as_str(), queue_id.as_str(), subscriber_id.get_value()
            );
        }

        if self
            .subscribers
            .iter()
            .any(|s| s.id.equals_to(subscriber_id))
        {
            panic!(
                "Somehow we generated the same ID {} for the new subscriber {}/{}",
                subscriber_id,
                topic_id.as_str(),
                queue_id.as_str()
            );
        }

        self.snapshot_id += 1;

        let mut evicted = Vec::new();
        if single_connection && !self.subscribers.is_empty() {
            std::mem::swap(&mut evicted, &mut self.subscribers);
        }

        self.subscribers
            .push(QueueSubscriber::new(subscriber_id, queue_id, session));

        evicted
    }

    pub fn get_amount(&self) -> usize {
        self.subscribers.len()
    }

    pub fn one_second_tick(&mut self) {
        for queue_subscriber in self.subscribers.iter_mut() {
            queue_subscriber.metrics.one_second_tick();
        }
    }

    fn resolve_subscriber_id_by_session_id(&self, session_id: SessionId) -> Option<SubscriberId> {
        self.subscribers
            .iter()
            .find(|s| s.session.session_id == session_id)
            .map(|s| s.id)
    }

    pub fn remove(&mut self, subscriber_id: SubscriberId) -> Option<QueueSubscriber> {
        let position = self
            .subscribers
            .iter()
            .position(|s| s.id.equals_to(subscriber_id))?;
        let result = self.subscribers.remove(position);
        self.last_unsubscribe = DateTimeAsMicroseconds::now();
        self.snapshot_id += 1;
        Some(result)
    }

    pub fn remove_by_session_id(&mut self, session_id: SessionId) -> Option<QueueSubscriber> {
        let subscriber_id = self.resolve_subscriber_id_by_session_id(session_id)?;
        self.remove(subscriber_id)
    }

    pub fn find_subscribers_dead_on_delivery(
        &self,
        max_delivery_duration: Duration,
    ) -> Vec<DeadSubscriber> {
        let mut result = vec![];
        for subscriber in self.subscribers.iter() {
            if let Some(duration) = subscriber.is_dead_on_delivery(max_delivery_duration) {
                result.push(DeadSubscriber::new(subscriber, duration));
            }
        }
        result
    }
}
