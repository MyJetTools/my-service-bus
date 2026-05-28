use std::sync::Arc;

use my_service_bus::{
    abstractions::publisher::MessageToPublish,
    tcp_contracts::{NodeSequenceNumber, NodeTopicPublish},
};
use tokio::sync::{Mutex, Notify};

use super::outbound_inner::{OutboundInner, PendingAck};

pub struct Ack {
    pub client_connection_id: i32,
    pub client_request_id: i64,
}

/// All node→master publish buffering. Single-locked Inner+Wrapper pattern so
/// the accept/flush/complete operations are atomic across the topic map and
/// in-flight state.
pub struct Outbound {
    inner: Mutex<OutboundInner>,
    /// Pulses whenever something changes that the flusher loop should re-check
    /// (new Publish accepted, in-flight completed, or connection re-opened).
    flush_signal: Notify,
}

impl Outbound {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(OutboundInner::new()),
            flush_signal: Notify::new(),
        })
    }

    pub async fn accept_client_publish(
        &self,
        topic_id: String,
        persist_immediately: bool,
        messages: Vec<MessageToPublish>,
        client_connection_id: i32,
        client_request_id: i64,
    ) {
        {
            let mut inner = self.inner.lock().await;
            inner.accept_client_publish(
                topic_id,
                persist_immediately,
                messages,
                client_connection_id,
                client_request_id,
            );
        }
        self.flush_signal.notify_one();
    }

    pub async fn try_take_flush(&self) -> Option<(NodeSequenceNumber, Vec<NodeTopicPublish>)> {
        self.inner.lock().await.try_take_flush()
    }

    pub async fn complete_in_flight(
        &self,
        sequence: NodeSequenceNumber,
    ) -> Result<Vec<Ack>, NodeSequenceNumber> {
        let acks = {
            let mut inner = self.inner.lock().await;
            inner.complete_in_flight(sequence)?
        };
        self.flush_signal.notify_one();
        Ok(acks.into_iter().map(Self::convert_ack).collect())
    }

    pub async fn drain_all_on_disconnect(&self) -> Vec<Ack> {
        let inner_acks = self.inner.lock().await.drain_all_on_disconnect();
        inner_acks.into_iter().map(Self::convert_ack).collect()
    }

    pub async fn wait_for_signal(&self) {
        self.flush_signal.notified().await
    }

    pub fn pulse(&self) {
        self.flush_signal.notify_one();
    }

    fn convert_ack(ack: PendingAck) -> Ack {
        Ack {
            client_connection_id: ack.client_connection_id,
            client_request_id: ack.client_request_id,
        }
    }
}
