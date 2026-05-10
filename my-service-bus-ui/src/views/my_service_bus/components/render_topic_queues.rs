use dioxus::prelude::*;

use crate::components::ui::{
    icon_plug, BtnVariant, BtnSize, Badge, Button, Tone,
};
use crate::dialogs::DialogState;
use crate::models::*;
use crate::views::my_service_bus::state::MySbState;

pub fn render_topic_queues(
    data: &MySbHttpContract,
    topic: &TopicHttpModel,
    cs: Signal<MySbState>,
) -> Element {
    let _ = cs;

    let Some(queues) = data.queues.get(&topic.id) else {
        return rsx! {
            div { class: "empty-state",
                style: "margin:20px auto; text-align:center;",
                h4 { "No queues" }
                p { "This topic has no active queues." }
            }
        };
    };

    if queues.queues.is_empty() {
        return rsx! {
            div { class: "empty-state",
                style: "margin:20px auto; text-align:center;",
                h4 { "No queues" }
                p { "This topic has no active queues." }
            }
        };
    }

    let to_render = queues.queues.iter().map(|topic_queue| {
        let queue_subs_count = topic
            .subscribers
            .iter()
            .filter(|s| s.queue_id == topic_queue.id)
            .count();

        // queue_type bits: bit0 = auto-delete, bit1 = single-connection
        let is_auto_delete = topic_queue.queue_type & 1 != 0;
        let is_single = topic_queue.queue_type & 2 != 0;

        let size_tone = if topic_queue.size > 1000 { Tone::Danger } else { Tone::Neutral };

        let subs_tone = if queue_subs_count > 0 { Tone::Success } else { Tone::Danger };
        let delete_mode_tone = if is_auto_delete { Tone::Info } else { Tone::Warning };
        let connect_mode_tone = if is_single { Tone::Pink } else { Tone::Accent };
        let delete_mode_label = if is_auto_delete { "auto-delete" } else { "permanent" };
        let connect_mode_label = if is_single { "single" } else { "multi" };

        // Show delete button only for permanent queues without active subscribers
        let can_delete = !is_auto_delete && queue_subs_count == 0;
        let topic_id_owned = topic.id.clone();
        let queue_id_owned = topic_queue.id.clone();

        let delete_button = if can_delete {
            rsx! {
                Button {
                    variant: BtnVariant::OutlineDanger,
                    size: BtnSize::Xs,
                    onclick: move |_| {
                        let t = topic_id_owned.clone();
                        let q = queue_id_owned.clone();
                        consume_context::<Signal<DialogState>>().set(DialogState::DeleteQueue {
                            topic_id: t.clone(),
                            queue_id: q.clone(),
                            on_ok: EventHandler::new(move |_| {
                                let t = t.clone();
                                let q = q.clone();
                                spawn(async move {
                                    if let Err(err) = crate::api::my_sb::delete_queue(&t, &q).await {
                                        dioxus_logger::tracing::error!("delete_queue failed: {err}");
                                    }
                                });
                            }),
                        });
                    },
                    "Delete"
                }
            }
        } else {
            rsx! {}
        };

        let render_subs = topic
            .subscribers
            .iter()
            .filter(|s| s.queue_id == topic_queue.id)
            .map(|itm| super::render_subscriber(data, itm));

        let queue_intervals = super::render_queues(&topic_queue.data);

        rsx! {
            div { class: "msb-queue",
                div { class: "msb-queue__head",
                    div { class: "msb-queue__title-row",
                        div { class: "msb-queue__title selectable", "{topic_queue.id}" }
                        {delete_button}
                    }
                    div { class: "msb-queue__badges",
                        Badge { tone: subs_tone,
                            span { class: "msb-conn__host-tag",
                                style: "padding:0;background:transparent;border:0;display:inline-flex;align-items:center;gap:3px;",
                                {icon_plug()}
                                "{queue_subs_count}"
                            }
                        }
                        Badge { tone: delete_mode_tone, "{delete_mode_label}" }
                        Badge { tone: connect_mode_tone, "{connect_mode_label}" }
                        Badge { tone: size_tone, mono: true,
                            "Sz {topic_queue.size}/{topic_queue.on_delivery}"
                        }
                        {queue_intervals}
                    }
                }
                if queue_subs_count > 0 {
                    div { class: "msb-queue__body", {render_subs} }
                }
            }
        }
    });

    rsx! { {to_render} }
}
