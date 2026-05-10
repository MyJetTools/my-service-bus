use std::time::Duration;

use dioxus::prelude::*;
use dioxus_utils::js::sleep;

use crate::components::ui::{
    status_item, status_item_end, status_live, vec_from, Badge, BtnSize, BtnVariant, Button,
    KpiCard, KpiTone, Sidebar, Sparkline, SparklineColor, SparklineKind, StatusValueTone, Tone,
    Topbar,
};
use crate::dialogs::{DialogState, RenderDialog};
use crate::models::{MySbHttpContract, TopicHttpModel};
use crate::utils::{format_mem, format_unix_micros};

use super::state::{MySbState, SidebarSection};

#[component]
pub fn RenderMyServiceBus() -> Element {
    let mut cs = use_signal(MySbState::default);
    let cs_ra = cs.read();

    start_background(cs, &cs_ra);

    let data = match get_data(&cs_ra) {
        Ok(data) => data,
        Err(err) => return err,
    };

    let filter = cs_ra.filter_string.clone();
    let active_section = cs_ra.active_section;
    let topics_len = data.topics.items.len();
    let sessions_len = data.sessions.items.len();
    let pages_total: usize = data.topics.items.iter().map(|t| t.pages.len()).sum();
    let now_ms = js_sys::Date::now();
    let is_live = cs_ra.last_updated_ms > 0.0
        && (now_ms - cs_ra.last_updated_ms) < 5_000.0
        && cs_ra.poll_failures == 0;

    let kpi_strip = render_kpi_strip(data, &cs_ra);
    let status_bar = render_status_bar(data, &cs_ra, is_live);

    let main_content = match active_section {
        SidebarSection::Topics => render_topics_section(data, &filter, cs),
        SidebarSection::Sessions => super::components::render_sessions(data, &filter),
        SidebarSection::Pages => render_pages_section(data),
    };

    drop(cs_ra);

    rsx! {
        RenderDialog {}
        div { class: "app-shell",
            div { class: "app-shell__sidebar",
                Sidebar {
                    active: active_section,
                    topics: topics_len,
                    sessions: sessions_len,
                    pages: pages_total,
                    is_live,
                    on_select: move |section: SidebarSection| {
                        cs.write().active_section = section;
                    },
                }
            }
            div { class: "app-shell__main",
                div { class: "app-shell__topbar",
                    Topbar {
                        section: active_section,
                        filter,
                        on_filter_change: move |s: String| cs.write().filter_string = s,
                    }
                }
                div { class: "app-shell__kpi", {kpi_strip} }
                div { class: "app-shell__content no-scrollbar", {main_content} }
            }
            div { class: "app-shell__statusbar", {status_bar} }
        }
    }
}

fn render_kpi_strip(data: &MySbHttpContract, cs_ra: &MySbState) -> Element {
    let bar = data.get_status_bar_calculated_values();
    let mem_pct = if data.system.totalmem > 0 {
        ((data.system.usedmem as f64 / data.system.totalmem as f64) * 100.0) as i32
    } else {
        0
    };
    let persist_tone = if bar.persist_queue >= 5000 {
        KpiTone::Danger
    } else if bar.persist_queue >= 1000 {
        KpiTone::Warning
    } else {
        KpiTone::Default
    };
    let mem_tone = if mem_pct >= 90 {
        KpiTone::Danger
    } else if mem_pct >= 75 {
        KpiTone::Warning
    } else {
        KpiTone::Default
    };

    rsx! {
        div { class: "msb-kpi-strip",
            KpiCard {
                label: "Msg / sec",
                value: format!("{}", bar.msg_per_sec),
                tone: KpiTone::Default,
                color: SparklineColor::Accent,
                history: vec_from(&cs_ra.kpi_history.msg_per_sec),
            }
            KpiCard {
                label: "Persist queue",
                value: format!("{}", bar.persist_queue),
                tone: persist_tone,
                color: if bar.persist_queue >= 5000 { SparklineColor::Red } else { SparklineColor::Amber },
                history: vec_from(&cs_ra.kpi_history.persist_queue),
            }
            KpiCard {
                label: "Sessions",
                value: format!("{}", data.sessions.items.len()),
                tone: KpiTone::Default,
                color: SparklineColor::Green,
                history: vec_from(&cs_ra.kpi_history.sessions),
            }
            KpiCard {
                label: "Memory used",
                value: format!("{}", mem_pct),
                unit: "%".to_string(),
                tone: mem_tone,
                color: if mem_pct >= 90 { SparklineColor::Red } else { SparklineColor::Accent },
                history: vec_from(&cs_ra.kpi_history.mem_used_pct),
            }
        }
    }
}

fn render_status_bar(data: &MySbHttpContract, cs_ra: &MySbState, is_live: bool) -> Element {
    let bar = data.get_status_bar_calculated_values();
    let persist_tone = if bar.persist_queue >= 5000 {
        StatusValueTone::Danger
    } else {
        StatusValueTone::Success
    };
    let mem_used = format_mem(data.system.usedmem);
    let mem_total = format_mem(data.system.totalmem);
    let mem_str = format!("{mem_used} / {mem_total}");

    let updated_str = if cs_ra.last_updated_ms > 0.0 {
        format_clock(cs_ra.last_updated_ms)
    } else {
        "--".to_string()
    };

    rsx! {
        div { class: "msb-statusbar",
            {status_live(is_live)}
            {status_item("Sessions", &data.sessions.items.len().to_string(), StatusValueTone::Default)}
            {status_item("Persist", &bar.persist_queue.to_string(), persist_tone)}
            {status_item("Msg/s", &bar.msg_per_sec.to_string(), StatusValueTone::Default)}
            {status_item("Req/s", &bar.packets_per_sec.to_string(), StatusValueTone::Default)}
            {status_item("Pages", &format_mem(bar.total_pages_size), StatusValueTone::Default)}
            {status_item("Mem", &mem_str, StatusValueTone::Default)}
            {status_item("sb", data.version.as_str(), StatusValueTone::Dim)}
            {status_item("persist", data.persistence_version.as_str(), StatusValueTone::Dim)}
            {status_item_end("Updated", &updated_str, StatusValueTone::Default)}
        }
    }
}

fn format_clock(ms: f64) -> String {
    let date = js_sys::Date::new(&ms.into());
    let h = date.get_hours();
    let m = date.get_minutes();
    let s = date.get_seconds();
    format!("{h:02}:{m:02}:{s:02}")
}

fn render_topics_section(
    data: &MySbHttpContract,
    filter: &str,
    cs: Signal<MySbState>,
) -> Element {
    let topics: Vec<&TopicHttpModel> = data
        .topics
        .items
        .iter()
        .filter(|topic| topic_matches_filter(data, topic, filter))
        .collect();

    if topics.is_empty() {
        return rsx! {
            div { class: "empty-state",
                h4 { "No topics" }
                p { "Nothing matches the current filter." }
            }
        };
    }

    let head = rsx! {
        div { class: "msb-col-head",
            div { "Topic" }
            div { "Connections" }
            div { "Queues" }
        }
    };

    let rows = topics.into_iter().map(|topic| render_topic_row(data, topic, cs));

    rsx! {
        div { class: "msb-topics",
            {head}
            {rows}
        }
    }
}

fn topic_matches_filter(data: &MySbHttpContract, topic: &TopicHttpModel, filter: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    if topic.id.to_lowercase().contains(filter) {
        return true;
    }
    if let Some(qs) = data.queues.get(&topic.id) {
        for q in &qs.queues {
            if q.id.to_lowercase().contains(filter) {
                return true;
            }
        }
    }
    false
}

fn render_topic_row(
    data: &MySbHttpContract,
    topic: &TopicHttpModel,
    cs: Signal<MySbState>,
) -> Element {
    let topic_connections = super::components::render_topic_connections(data, topic);
    let render_queues = super::components::render_topic_queues(data, topic, cs);
    let publish_history = topic.publish_history.clone();
    let max_throughput = publish_history.iter().map(|v| v.unsigned_abs() as i32).max().unwrap_or(0);

    let queues_for_topic = data.queues.get(&topic.id);
    let has_queues = queues_for_topic.map(|qs| !qs.queues.is_empty()).unwrap_or(false);
    let is_deleted = topic.deleted != 0;
    let can_delete_topic = topic.publishers.is_empty() && !has_queues;
    let topic_id_for_btn = topic.id.clone();
    let topic_id_for_restore = topic.id.clone();

    let row_actions = if is_deleted {
        let deleted_until = format_unix_micros(topic.deleted);
        rsx! {
            div { class: "msb-topic__row-actions",
                Badge { tone: Tone::Danger, "Deleted until {deleted_until}" }
                Button {
                    variant: BtnVariant::OutlineSuccess,
                    size: BtnSize::Xs,
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
            div { class: "msb-topic__row-actions",
                Button {
                    variant: BtnVariant::OutlineDanger,
                    size: BtnSize::Xs,
                    onclick: move |_| {
                        let topic_id = topic_id_for_btn.clone();
                        consume_context::<Signal<DialogState>>().set(DialogState::DeleteTopic {
                            topic_id: topic_id.clone(),
                            on_ok: EventHandler::new(move |iso: String| {
                                let t = topic_id.clone();
                                spawn(async move {
                                    if let Err(err) = crate::api::my_sb::delete_topic(&t, &iso).await {
                                        dioxus_logger::tracing::error!("delete_topic failed: {err}");
                                    }
                                });
                            }),
                        });
                    },
                    "Delete topic"
                }
            }
        }
    } else {
        rsx! {}
    };

    let pages = topic.pages.iter().map(|page| {
        super::components::render_page(page.id, page.amount, page.size, &page.sub_pages)
    });

    let dot_class = if is_deleted {
        "msb-topic__dot is-deleted"
    } else if topic.messages_per_src > 0 || !topic.publishers.is_empty() {
        "msb-topic__dot is-active"
    } else {
        "msb-topic__dot"
    };

    let persist_value_class = if topic.persist_size > 0 { "value is-warning" } else { "value" };
    let msg_class = if topic.messages_per_src > 0 { "value is-active" } else { "value" };

    rsx! {
        div { class: "msb-topic",
            div { class: "msb-topic__head",
                div { class: "msb-topic__title",
                    span { class: "{dot_class}" }
                    span { class: "selectable", "{topic.id}" }
                }
                div { class: "msb-topic__stat-grid",
                    span { class: "label", "MsgId" }
                    span { class: "value", "{topic.message_id}" }
                    span { class: "label", "Msg/s" }
                    span { class: "{msg_class}", "{topic.messages_per_src}" }
                    span { class: "label", "Req/s" }
                    span { class: "value", "{topic.packet_per_sec}" }
                    span { class: "label", "Persist q" }
                    span { class: "{persist_value_class}", "{topic.persist_size}" }
                }
                div { class: "msb-spark-card",
                    div { class: "msb-spark-row",
                        span { "Throughput" }
                        span { class: "max", "max {max_throughput}" }
                    }
                    Sparkline {
                        kind: SparklineKind::Area,
                        width: 240,
                        height: 36,
                        color: SparklineColor::Accent,
                        data: publish_history,
                    }
                }
                div { class: "msb-topic__pages", {pages} }
                {row_actions}
            }
            div { {topic_connections} }
            div { {render_queues} }
        }
    }
}

fn render_pages_section(data: &MySbHttpContract) -> Element {
    let mut blocks: Vec<Element> = Vec::new();
    for topic in &data.topics.items {
        if topic.pages.is_empty() {
            continue;
        }
        let pages = topic
            .pages
            .iter()
            .map(|p| super::components::render_page(p.id, p.amount, p.size, &p.sub_pages));
        blocks.push(rsx! {
            div { class: "msb-card",
                div { class: "msb-card__header",
                    div { class: "msb-topic__title",
                        span { class: "msb-topic__dot is-active" }
                        span { class: "selectable", "{topic.id}" }
                    }
                }
                div { class: "msb-card__body",
                    div { class: "msb-topic__pages", {pages} }
                }
            }
        });
    }

    if blocks.is_empty() {
        return rsx! {
            div { class: "empty-state",
                h4 { "No pages" }
                p { "No persisted pages on disk yet." }
            }
        };
    }

    rsx! {
        div { class: "msb-topics", {blocks.into_iter()} }
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
                    let mut w = cs.write();
                    w.poll_failures = 0;
                    w.last_updated_ms = js_sys::Date::now();
                    w.push_kpi_sample(&data);
                    w.data.set_loaded(data);
                }
                Err(err) => {
                    let mut w = cs.write();
                    w.poll_failures = w.poll_failures.saturating_add(1);
                    if w.poll_failures >= 3 {
                        w.kpi_history.clear();
                    }
                    w.data.set_error(err);
                }
            }

            sleep(Duration::from_secs(1)).await;
        }
    });
}

fn get_data(cs_ra: &MySbState) -> Result<&MySbHttpContract, Element> {
    match cs_ra.data.as_ref() {
        dioxus_utils::RenderState::None => Err(crate::components::render_loading()),
        dioxus_utils::RenderState::Loading => Err(crate::components::render_loading()),
        dioxus_utils::RenderState::Loaded(data) => Ok(data),
        dioxus_utils::RenderState::Error(err) => {
            // If we still have a previous snapshot, render it; otherwise show the error.
            if cs_ra.data.has_value() {
                if let Some(prev) = cs_ra.data.try_unwrap_as_loaded() {
                    return Ok(prev);
                }
            }
            Err(crate::components::render_error(err))
        }
    }
}

