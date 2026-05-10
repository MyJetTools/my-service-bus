use dioxus::prelude::*;

use crate::models::*;

pub fn render_topic_queues(data: &MySbHttpContract, topic: &TopicHttpModel) -> Element {
    let queues = data.queues.get(&topic.id);

    let Some(queues) = queues else {
        return rsx! {};
    };

    let to_render = queues.queues.iter().map(|topic_queue| {
        let subs_amount = topic.subscribers.len();
        let subscribers_amount = if subs_amount > 0 {
            rsx! {
                span { class: "badge text-bg-success",
                    img {
                        style: "height:9px",
                        src: asset!("/assets/ico/plug.svg"),
                    }
                    {subs_amount.to_string()}
                }
            }
        } else {
            rsx! {
                span { class: "badge text-bg-danger",
                    img {
                        style: "height:9px",
                        src: asset!("/assets/ico/plug.svg"),
                    }
                    {subs_amount.to_string()}
                }
            }
        };

        let render_subs = topic
            .subscribers
            .iter()
            .filter(|s| s.queue_id == topic_queue.id)
            .map(|itm| super::render_subscriber(data, itm));

        let q_size_badge = if topic_queue.size > 1000 {
            rsx! {
                span { class: "badge text-bg-danger",
                    "Size: {topic_queue.size}/{topic_queue.on_delivery}"
                }
            }
        } else {
            rsx! {
                span { class: "badge text-bg-success",
                    "Size: {topic_queue.size}/{topic_queue.on_delivery}"
                }
            }
        };

        // queue_type bits: bit0 = auto-delete, bit1 = single-connection
        // 0=permanent+multi, 1=auto-delete+multi, 2=permanent+single, 3=auto-delete+single
        let is_auto_delete = topic_queue.queue_type & 1 != 0;
        let is_single = topic_queue.queue_type & 2 != 0;

        let delete_mode = if is_auto_delete {
            rsx! {
                span { class: "badge text-bg-success", "auto-delete" }
            }
        } else {
            rsx! {
                span { class: "badge text-bg-warning", "permanent" }
            }
        };

        let connect_mode = if is_single {
            rsx! {
                span { class: "badge text-bg-primary", "single-connect" }
            }
        } else {
            rsx! {
                span { class: "badge text-bg-info", "multi-connect" }
            }
        };

        let queue_type = rsx! {
            {delete_mode}
            {connect_mode}
        };

        let queue = super::render_queues(&topic_queue.data);

        rsx! {
            div { class: "topic-queue",

                div {
                    {topic_queue.id.as_str()}
                    div {
                        {subscribers_amount}
                        {queue_type}
                        {q_size_badge}
                        {queue}
                    }
                }
                div { {render_subs} }
            }

        }
    });

    rsx! {
        {to_render}
    }
}
