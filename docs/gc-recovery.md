# GC and Cache Recovery

This document describes how the message-page garbage collector behaves and what
recovery semantics apply to **persist=true** vs **persist=false** topics.

## Storage layout

- A **Page** holds 100,000 messages. It is the granularity used by the
  persistence layer and by the UI for visualization.
- A **SubPage** holds 1,000 messages. It is the in-memory unit for allocation,
  GC, and persistence I/O. One Page contains 100 SubPages.

Splitting Pages into SubPages was introduced because allocating a full
100k-slot Page just to hold a handful of live messages was wasteful. With
SubPages we keep memory aligned to actual cache pressure.

## GC entry points

`TopicInner::gc()` runs every 3 seconds (`GcTimer`) and right after
`mark_messages_as_persisted`. Under the topic lock it:

1. **Fast-path: no queues** — if `self.queues` is empty, no consumer can ever
   reach any message, so we drop every SubPage except the current one (the one
   matching `self.message_id`) via `MessagesPageList::gc_all_except`. SubPages
   that still hold pending-persist messages are kept (`is_ready_to_gc(&[])`
   guard) so on-disk durability is not violated.

2. **Regular path** — compute `min_message_id` and `active_sub_pages`, then:
   - `gc_pages` removes any SubPage that is not in `active_sub_pages` and has
     no pending-persist content.
   - `gc_messages` removes individual messages with `id < min_message_id`
     within each kept SubPage.

The set of "active" SubPages is computed differently for the two persistence
modes — see below.

## Persistence modes

MyServiceBus supports two semantically distinct modes per topic, and the GC
recovery behavior differs between them.

### persist=true

The persistence service holds the durable copy of every message. If a SubPage
is evicted from the cache while it is still referenced by a queue, the next
delivery attempt simply re-loads it from disk via `LoadSubPageScheduler` →
`load_page`. Lost cache entries are recoverable.

Therefore `get_active_sub_pages` only protects:

- The current SubPage (`self.message_id` → `SubPageId`).
- The SubPage of `queue.get_min_msg_id()` for each queue.

Higher SubPages within a queue's interval are allowed to be GC'd; they will be
re-fetched on demand. This keeps cache pressure low.

### persist=false

There is no durable copy. A SubPage that is evicted is gone forever — its
content cannot be recovered. We therefore protect **every** SubPage that any
live consumer might still need:

- The current SubPage.
- For every queue, every SubPage touched by any interval in its
  `QueueWithIntervals` (not just the minimum).
- For every subscriber on every queue, every SubPage touched by intervals in
  `subscriber.get_messages_on_delivery()`.

`ActiveSubPages::add_intervals` walks each `QueueIndexRange` and registers
every SubPage from `from_id`'s SubPage to `to_id`'s SubPage inclusive.

The semantic intent is **TCP-like**: while a session is active no message is
silently dropped; on disconnect/restart we accept loss.

## Cache-miss during delivery

When `compile_package` finds the SubPage missing from the cache (`pages.get_mut`
returns `None`) or finds a specific message id `NotLoaded`, behavior also
depends on the persistence mode:

- **persist=true** — schedule `load_sub_page` and bail out of the current
  delivery iteration. The page will be reloaded from disk and the next
  delivery cycle will see it.
- **persist=false** — there is no source of truth for the missing content, so
  reloading would only produce a SubPage of `MySbCachedMessage::Missing(...)`
  entries (the "zombie page" symptom: `Amount:1000; Size:0`). Instead we:
  1. Find the next existing SubPage in the cache via
     `MessagesPageList::find_next_existing_sub_page`. If none, fall back to
     the current SubPage (`self.message_id`).
  2. Drain the queue of every id below that target's first message id
     (`drain_queue_below`).
  3. Continue the delivery loop. The subscriber "skips ahead" past the lost
     range — TCP-style.

For `NotLoaded` (the SubPage exists but the specific id was already GC'd from
within it), persist=false simply skips the id — it was already dequeued.

## Known gap: stale Permanent queues with no subscribers (persist=false)

The cache-miss skip path requires a subscriber to actually attempt delivery.
For `Permanent` / `PermanentWithSingleConnection` queues that survive without
any subscriber (so `gc_queues_with_no_subscribers` cannot delete them — that
helper only removes `DeleteOnDisconnect` queues after the grace period), a
queue can keep referencing a "zombie" SubPage forever:

- Topic restarts with a queue snapshot containing old message ids.
- persist=false, so message content was never persisted; only the queue
  intervals were.
- No subscriber attaches → no delivery attempt → cache-miss path never runs →
  the queue is never drained → `add_intervals` keeps the dead SubPage marked
  active → GC cannot evict it.

Possible follow-ups:

- In `gc()` for `persist=false`, for any queue that currently has zero
  subscribers, drain its `queue` below the first existing SubPage's
  first_message_id. Safe by construction (persist=false + no subscriber → the
  ids are unreachable anyway).
- Or expose an admin operation that purges stale queue content for
  `persist=false` topics.

## Files

- [src/topics/topic_inner.rs](../src/topics/topic_inner.rs) —
  `gc`, `get_active_sub_pages`, `get_min_message_id`.
- [src/messages_page/messages_page_list.rs](../src/messages_page/messages_page_list.rs) —
  `gc_pages`, `gc_messages`, `gc_all_except`, `find_next_existing_sub_page`.
- [src/messages_page/active_sub_pages.rs](../src/messages_page/active_sub_pages.rs) —
  `ActiveSubPages`, `add_intervals`.
- [src/sub_page/sub_page.rs](../src/sub_page/sub_page.rs) — `is_ready_to_gc`.
- [src/operations/delivery/delivery.rs](../src/operations/delivery/delivery.rs) —
  cache-miss handling, `drain_queue_below`.
- [src/background/gc_timer.rs](../src/background/gc_timer.rs) — periodic
  invocation.
- [src/operations/page_loader/operations.rs](../src/operations/page_loader/operations.rs) —
  `load_page` (creates `Missing` entries when content is absent on disk).

## Related tests

- `topics::topic_inner::tests::persist_false_protects_all_sub_pages_in_queue_intervals`
- `topics::topic_inner::tests::persist_true_only_protects_min_sub_page`
- `topics::topic_inner::tests::no_queues_only_current_sub_page_survives_gc`
- `sub_page::sub_page::tests::is_not_ready_to_gc_while_persist_queue_has_items`
- `operations::gc_message_pages::tests::test_that_we_do_not_gc_messages_which_are_on_delivery`
