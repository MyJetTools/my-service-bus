use dioxus::prelude::*;

use crate::models::*;
use crate::views::my_service_bus::state::MySbState;

pub fn render_topic_queues(
    data: &MySbHttpContract,
    topic: &TopicHttpModel,
    mut cs: Signal<MySbState>,
) -> Element {
    let queues = data.queues.get(&topic.id);

    let Some(queues) = queues else {
        return rsx! {};
    };

    let to_render = queues.queues.iter().map(|topic_queue| {
        // subscribers attached to THIS queue specifically
        let queue_subs_count = topic
            .subscribers
            .iter()
            .filter(|s| s.queue_id == topic_queue.id)
            .count();

        let subscribers_amount = if queue_subs_count > 0 {
            rsx! {
                span { class: "badge text-bg-success",
                    img {
                        style: "height:9px",
                        src: asset!("/assets/ico/plug.svg"),
                    }
                    {queue_subs_count.to_string()}
                }
            }
        } else {
            rsx! {
                span { class: "badge text-bg-danger",
                    img {
                        style: "height:9px",
                        src: asset!("/assets/ico/plug.svg"),
                    }
                    {queue_subs_count.to_string()}
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

        // Show delete button only for permanent queues without active subscribers
        let can_delete = !is_auto_delete && queue_subs_count == 0;
        let topic_id_owned = topic.id.clone();
        let queue_id_owned = topic_queue.id.clone();
        let delete_button = if can_delete {
            rsx! {
                button {
                    class: "btn btn-sm btn-outline-danger ms-2",
                    style: "padding: 0 6px; font-size: 11px;",
                    onclick: move |_| {
                        cs.write().delete_queue_dialog =
                            Some((topic_id_owned.clone(), queue_id_owned.clone()));
                    },
                    "Delete"
                }
            }
        } else {
            rsx! {}
        };

        let queue_type = rsx! {
            {delete_mode}
            {connect_mode}
        };

        let queue = super::render_queues(&topic_queue.data);

        rsx! {
            div { class: "topic-queue",

                div {
                    span { class: "selectable", {topic_queue.id.as_str()} }
                    {delete_button}
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
