## Garbage Collector

This service runs GC in two layers whenever the background GC timer ticks:

1) Message-level GC  
- Each topic computes the minimal message_id still needed (minimum across publisher cursor, every queue head, and every subscriber cursor).  
- For each sub-page, messages with id < that minimal id are eligible **only if they are not awaiting persistence** (`to_persist` queue).  
- Eligible messages are removed from the sub-page; size counters are updated.  
- Persistence guard: if a message_id is still in `to_persist`, it is kept until marked persisted.  
References: `TopicInner::get_min_message_id`, `TopicInner::gc`, `SubPageInner::gc_messages`, `SubPageInner::message_can_be_gc`.

2) Page-level GC  
- Active sub-pages are determined (current producing page plus any page that holds the head message for each queue/subscriber).  
- A sub-page can be dropped only if:  
  - It has no messages queued for persistence, and  
  - It is not in the active set, and  
  - It is empty after message-level GC (for the `gc_messages` pass) or explicitly marked ready (`gc_pages`).  
- Pages meeting the conditions are removed from `MessagesPageList`.  
References: `TopicInner::get_active_sub_pages`, `MessagesPageList::gc_pages`, `MessagesPageList::gc_messages`, `SubPage::is_ready_to_be_gc`.

Lifecycle trigger  
- `GcTimer` runs periodically, iterates topics, calls `topic_data.gc()`, then applies queue deletion rules and connection GC. This is what kicks off both message- and page-level GC.
