use ahash::AHashMap;

use my_service_bus::{
    abstractions::publisher::MessageToPublish,
    tcp_contracts::{NodeSequenceNumber, NodeTopicPublish},
};

/// Identifies one ack a node owes a downstream client once master confirms
/// the messages were accepted.
#[derive(Debug, Clone)]
pub(super) struct PendingAck {
    /// my-tcp-sockets `ConnectionId` of the client that issued the Publish.
    pub client_connection_id: i32,
    /// The `request_id` the client put into its Publish packet — we must echo
    /// it back unchanged in the PublishResponse so the client correlates.
    pub client_request_id: i64,
}

#[derive(Debug, Default)]
struct TopicAccumulator {
    /// OR of `persist_immediately` across all client Publish batches accepted
    /// for this topic since the last flush.
    persist_immediately: bool,
    messages: Vec<MessageToPublish>,
}

pub(super) struct InFlight {
    pub sequence: NodeSequenceNumber,
    pub pending_acks: Vec<PendingAck>,
}

pub(super) struct OutboundInner {
    /// Buffered Publishes that haven't been sent to master yet, grouped by
    /// topic. Drained atomically on each flush.
    topics: AHashMap<String, TopicAccumulator>,
    /// Acks owed once the in-flight NodePublish (if any) gets its response.
    /// These came from the client Publish packets that fed the current flush
    /// — they live here, not in `topics`, because by the time we hold
    /// `in_flight` the topic buffer is already drained.
    next_sequence: NodeSequenceNumber,
    in_flight: Option<InFlight>,
    /// Acks accumulated for the *next* flush (clients that issued Publish
    /// while we were already in-flight). Moves into the new `InFlight` on the
    /// next drain.
    next_flush_acks: Vec<PendingAck>,
}

impl OutboundInner {
    pub(super) fn new() -> Self {
        Self {
            topics: AHashMap::new(),
            next_sequence: 0,
            in_flight: None,
            next_flush_acks: Vec::new(),
        }
    }

    pub(super) fn accept_client_publish(
        &mut self,
        topic_id: String,
        persist_immediately: bool,
        messages: Vec<MessageToPublish>,
        client_connection_id: i32,
        client_request_id: i64,
    ) {
        let acc = self.topics.entry(topic_id).or_default();
        if persist_immediately {
            acc.persist_immediately = true;
        }
        acc.messages.extend(messages);
        self.next_flush_acks.push(PendingAck {
            client_connection_id,
            client_request_id,
        });
    }

    /// Returns `Some((sequence, topics))` if a flush should be sent right now,
    /// `None` if either the channel is busy with an in-flight ack or there's
    /// nothing buffered.
    pub(super) fn try_take_flush(&mut self) -> Option<(NodeSequenceNumber, Vec<NodeTopicPublish>)> {
        if self.in_flight.is_some() {
            return None;
        }
        if self.topics.is_empty() {
            return None;
        }
        let topics: Vec<NodeTopicPublish> = self
            .topics
            .drain()
            .map(|(topic_id, acc)| NodeTopicPublish {
                topic_id,
                persist_immediately: acc.persist_immediately,
                data_to_publish: acc.messages,
            })
            .collect();
        self.next_sequence += 1;
        let sequence = self.next_sequence;
        let pending_acks = std::mem::take(&mut self.next_flush_acks);
        self.in_flight = Some(InFlight {
            sequence,
            pending_acks,
        });
        Some((sequence, topics))
    }

    /// Called when master responds with NodePublishResponse for the in-flight
    /// sequence. Returns the pending acks the caller now owes to clients.
    /// Returns `Err(())` if the sequence doesn't match — caller should treat
    /// this as a protocol corruption and reset the connection.
    pub(super) fn complete_in_flight(
        &mut self,
        sequence: NodeSequenceNumber,
    ) -> Result<Vec<PendingAck>, NodeSequenceNumber> {
        match self.in_flight.take() {
            Some(in_flight) if in_flight.sequence == sequence => Ok(in_flight.pending_acks),
            Some(in_flight) => {
                let expected = in_flight.sequence;
                // Put it back so we don't lose the pending acks silently if
                // caller decides to keep going.
                self.in_flight = Some(in_flight);
                Err(expected)
            }
            None => Err(0),
        }
    }

    /// Drop everything buffered (used on master-connection loss). Returns all
    /// acks that won't be honored (in-flight + queued) so the caller can fail
    /// the corresponding clients explicitly.
    pub(super) fn drain_all_on_disconnect(&mut self) -> Vec<PendingAck> {
        self.topics.clear();
        let mut acks = std::mem::take(&mut self.next_flush_acks);
        if let Some(in_flight) = self.in_flight.take() {
            acks.extend(in_flight.pending_acks);
        }
        acks
    }
}
