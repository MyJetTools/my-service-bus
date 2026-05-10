use std::time::Duration;

use dioxus::prelude::*;
use dioxus_utils::js::sleep;

use crate::models::MySbHttpContract;
use crate::utils::format_unix_micros;

use super::state::*;

#[component]
pub fn RenderMyServiceBus() -> Element {
    let mut cs = use_signal(|| MySbState::default());
    let cs_ra = cs.read();

    start_background(cs, &cs_ra);

    let data = match get_data(&cs_ra) {
        Ok(data) => data,
        Err(err) => return err,
    };

    let filter = cs_ra.filter_string.clone();

    let topics_iter = data.topics.items.iter().filter(|topic| {
        if filter.is_empty() {
            return true;
        }
        if topic.id.to_lowercase().contains(&filter) {
            return true;
        }
        if let Some(qs) = data.queues.get(&topic.id) {
            for q in &qs.queues {
                if q.id.to_lowercase().contains(&filter) {
                    return true;
                }
            }
        }
        false
    });

    let topics_to_render = topics_iter.map(|topic| {
        let topic_connections = super::components::render_topic_connections(data, topic);

        let graph = super::components::render_graph(true, topic.publish_history.as_slice());

        let rendered_pages = topic.pages.iter().map(|page| {
            super::components::render_page(page.id, page.amount, page.size, &page.sub_pages)
        });

        let render_queues = super::components::render_topic_queues(data, topic, cs);

        // Delete-topic button shown only for orphan topics: no publishers AND no queues.
        let queues_for_topic = data.queues.get(&topic.id);
        let has_queues = queues_for_topic
            .map(|qs| !qs.queues.is_empty())
            .unwrap_or(false);
        let is_deleted = topic.deleted != 0;
        let can_delete_topic = topic.publishers.is_empty() && !has_queues;
        let topic_id_for_btn = topic.id.clone();
        let topic_id_for_restore = topic.id.clone();
        let delete_topic_button = if is_deleted {
            let deleted_until = format_unix_micros(topic.deleted);
            rsx! {
                div {
                    style: "display:flex; flex-direction: column; gap:4px; margin-top:4px;",
                    span {
                        class: "badge",
                        style: "background:#a02020; color:white; padding:2px 6px; font-size:10px; align-self: flex-start;",
                        "Deleted until {deleted_until}"
                    }
                    button {
                        class: "btn btn-sm btn-outline-success",
                        style: "padding: 0 6px; font-size: 11px;",
                        onclick: move |_| {
                            let t = topic_id_for_restore.clone();
                            spawn(async move {
                                if let Err(err) = crate::api::my_sb::restore_topic(&t).await {
                                    dioxus_logger::tracing::error!("restore_topic failed: {err}");
                                }
                            });
                        },
                        "Restore"
                    }
                }
            }
        } else if can_delete_topic {
            rsx! {
                button {
                    class: "btn btn-sm btn-outline-danger",
                    style: "padding: 0 6px; font-size: 11px; margin-top: 4px;",
                    onclick: move |_| {
                        cs.write().delete_topic_dialog = Some(topic_id_for_btn.clone());
                    },
                    "Delete topic"
                }
            }
        } else {
            rsx! {}
        };

        rsx! {
            tr {
                td {
                    div { style: "font-size:16px",
                        b { class: "selectable", {topic.id.as_str()} }
                    }
                    div { style: "font-size:10px", "MsgId: {topic.message_id.to_string()}" }
                    div { style: "font-size:10px", "Msg/sec: {topic.messages_per_src.to_string()}" }
                    div { style: "font-size:10px", "Req/sec: {topic.packet_per_sec.to_string()}" }
                    div { style: "font-size:10px", "Persist q: {topic.persist_size.to_string()}" }
                    div { {graph} }
                    div { {rendered_pages} }
                    {delete_topic_button}

                }
                td { {topic_connections} }
                td { {render_queues} }
            }
        }
    });

    let sessions = super::components::render_sessions(data, &cs_ra.filter_string);

    let status_bar = data.get_status_bar_calculated_values();

    let persist_queue = if status_bar.persist_queue < 5000 {
        rsx! {
            b { style: "color:green", "{status_bar.persist_queue}" }
        }
    } else {
        rsx! {
            b { style: "color:red", "{status_bar.persist_queue}" }
        }
    };

    let mem_used = crate::utils::format_mem(data.system.usedmem);
    let mem_total = crate::utils::format_mem(data.system.totalmem);

    let filter_value = cs_ra.filter_string.clone();
    let queue_dialog = cs_ra.delete_queue_dialog.clone();
    let topic_dialog = cs_ra.delete_topic_dialog.clone();

    rsx! {
        if let Some((dlg_topic, dlg_queue)) = queue_dialog {
            {render_delete_dialog(cs, dlg_topic, dlg_queue)}
        }
        if let Some(dlg_topic) = topic_dialog {
            {render_delete_topic_dialog(cs, dlg_topic)}
        }
        div { class: "layout-with-status-bar",

            div { class: "no-scrollbar", style: "overflow-y:auto",

                table {
                    style: "margin:0",
                    class: "table table-striped table-dark sticky-thead",
                    thead {
                        tr {
                            th { "Topics" }
                            th { "Topic Connections" }
                            th {
                                div { class: "queues-header",
                                    span { "Queues" }
                                    input {
                                        r#type: "text",
                                        class: "header-search",
                                        placeholder: "Filter topic / queue / session…",
                                        value: "{filter_value}",
                                        oninput: move |e| {
                                            cs.write().filter_string = e.value().to_lowercase();
                                        },
                                    }
                                }
                            }
                        }
                    }
                    tbody { {topics_to_render} }
                }

                h1 { style: "background:black; color:white ;margin:0; padding:10px",
                    "Sessions"
                }

                table {
                    style: "margin:0",
                    class: "table table-striped table-dark",
                    thead {
                        tr {
                            th { "Id" }
                            th { "Info" }
                            th { "Publishers" }
                            th { "Subscribers" }
                        }
                    }
                    tbody { {sessions} }
                }
            }
            div { class: "status-bar",
                div { class: "item", "Sessions: {data.sessions.items.len()}" }
                div { class: "item",
                    "Persist q: "
                    {persist_queue}
                }

                div { class: "item", "Msg/sec: {status_bar.msg_per_sec}" }
                div { class: "item", "Req/sec: {status_bar.packets_per_sec}" }
                div { class: "item", "Total pages size: {status_bar.total_pages_size}" }

                div { class: "item", "Mem: {mem_used} / {mem_total}" }
                div { class: "item", "sb-version: {data.version.as_str()}" }
                div { class: "item", "persistence: {data.persistence_version.as_str()}" }
            }
        }

    }
}

fn start_background(mut cs: Signal<MySbState>, cs_ra: &MySbState) {
    if cs_ra.started {
        return;
    }
    spawn(async move {
        cs.write().started = true;

        loop {
            match crate::api::my_sb::get_data().await {
                Ok(data) => {
                    cs.write().data.set_loaded(data);
                }
                Err(err) => {
                    cs.write().data.set_error(err);
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    });
}

fn get_data(cs_ra: &MySbState) -> Result<&MySbHttpContract, Element> {
    match cs_ra.data.as_ref() {
        dioxus_utils::RenderState::None => Err(rsx! {}),
        dioxus_utils::RenderState::Loading => Err(crate::components::render_loading()),
        dioxus_utils::RenderState::Loaded(data) => Ok(data),
        dioxus_utils::RenderState::Error(err) => Err(crate::components::render_error(err)),
    }
}

fn render_delete_topic_dialog(cs: Signal<MySbState>, topic_id: String) -> Element {
    rsx! {
        DeleteTopicDialog { cs, topic_id }
    }
}

#[component]
fn DeleteTopicDialog(cs: Signal<MySbState>, topic_id: String) -> Element {
    let mut hard_24h = use_signal(|| false);
    let mut cs = cs;
    let topic_label = topic_id.clone();
    let topic_for_delete = topic_id.clone();
    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-card",
                h3 { style: "margin-top:0", "Confirm" }
                p {
                    "Confirm to delete topic "
                    b { "{topic_label}" }
                    "?"
                }
                div { style: "display:flex; flex-direction: column; gap:6px; margin: 12px 0;",
                    label { style: "cursor: pointer;",
                        input {
                            r#type: "radio",
                            name: "hard_delete_moment",
                            checked: !hard_24h(),
                            onchange: move |_| hard_24h.set(false),
                        }
                        " Now (immediate hard delete)"
                    }
                    label { style: "cursor: pointer;",
                        input {
                            r#type: "radio",
                            name: "hard_delete_moment",
                            checked: hard_24h(),
                            onchange: move |_| hard_24h.set(true),
                        }
                        " After 24 hours (allows restore)"
                    }
                }
                div { style: "display:flex; justify-content:flex-end; gap:10px; margin-top:16px",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            cs.write().delete_topic_dialog = None;
                        },
                        "Cancel"
                    }
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| {
                            let now_ms = js_sys::Date::now();
                            let target_ms = if hard_24h() { now_ms + 86_400_000.0 } else { now_ms };
                            let date = js_sys::Date::new(&target_ms.into());
                            let iso: String = date.to_iso_string().into();
                            let t = topic_for_delete.clone();
                            let mut cs = cs;
                            spawn(async move {
                                if let Err(err) = crate::api::my_sb::delete_topic(&t, &iso).await {
                                    dioxus_logger::tracing::error!("delete_topic failed: {err}");
                                }
                                cs.write().delete_topic_dialog = None;
                            });
                        },
                        "Delete"
                    }
                }
            }
        }
    }
}

fn render_delete_dialog(
    mut cs: Signal<MySbState>,
    topic_id: String,
    queue_id: String,
) -> Element {
    let cancel_topic = topic_id.clone();
    let cancel_queue = queue_id.clone();
    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-card",
                h3 { style: "margin-top:0", "Confirm" }
                p {
                    "Confirm to delete queue "
                    b { "{cancel_topic}/{cancel_queue}" }
                    "?"
                }
                div { style: "display:flex; justify-content:flex-end; gap:10px; margin-top:16px",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            cs.write().delete_queue_dialog = None;
                        },
                        "Cancel"
                    }
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| {
                            let t = topic_id.clone();
                            let q = queue_id.clone();
                            let mut cs = cs;
                            spawn(async move {
                                if let Err(err) = crate::api::my_sb::delete_queue(&t, &q).await {
                                    dioxus_logger::tracing::error!("delete_queue failed: {err}");
                                }
                                cs.write().delete_queue_dialog = None;
                            });
                        },
                        "Delete"
                    }
                }
            }
        }
    }
}
