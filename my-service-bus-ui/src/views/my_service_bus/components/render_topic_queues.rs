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

        let queue_type = match topic_queue.queue_type {
            0 => rsx! {
                span { class: "badge text-bg-warning", "permanent" }
            },
            1 => rsx! {
                span { class: "badge text-bg-success", "auto-delete" }
            },
            2 => {
                rsx! {
                    span { class: "badge text-bg-primary", "permanent-single-connect" }
                }
            }

            _ => rsx! {
                span { class: "badge text-bg-danger", "unknown" }
            },
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
