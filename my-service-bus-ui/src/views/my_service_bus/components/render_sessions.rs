use dioxus::prelude::*;

use crate::components::ui::{Badge, StatusPill, Tone};
use crate::models::*;
use crate::utils::format_mem;

pub fn render_sessions(data: &MySbHttpContract, filter_string: &str) -> Element {
    if data.sessions.items.is_empty() {
        return rsx! {
            div { class: "empty-state",
                h4 { "No sessions" }
                p { "No active sessions." }
            }
        };
    }

    let filter_active = !filter_string.is_empty();
    let items = data.sessions.items.iter().map(|session| {
        let is_match = filter_active && session.filter_me(filter_string);
        let class = if !filter_active {
            "msb-session"
        } else if is_match {
            "msb-session is-match"
        } else {
            "msb-session is-dim"
        };
        render_one(data, session, class)
    });

    rsx! {
        div { class: "msb-topics", {items} }
    }
}

fn render_one(data: &MySbHttpContract, session: &MySbSessionHttpModel, row_class: &str) -> Element {
    let session_type = match session.get_session_type() {
        SessionType::Tcp => rsx! { StatusPill { tone: Tone::Success, dot: true, "TCP" } },
        SessionType::Http => rsx! { StatusPill { tone: Tone::Warning, dot: true, "HTTP" } },
    };

    let r_size = format_mem(session.read_size);
    let w_size = format_mem(session.written_size);
    let r_p_s = format_mem(session.read_per_sec);
    let w_p_s = format_mem(session.written_per_sec);

    let (publishers, subscribers) = data.get_publishers_and_subscribers(session.id);

    let pub_chips = publishers.into_iter().map(|(name, active)| {
        let tone = if active > 0 { Tone::Success } else { Tone::Neutral };
        rsx! { Badge { tone, "{name}" } }
    });

    let sub_chips = subscribers.into_iter().map(|(topic, queue, active)| {
        let tone = if active > 0 { Tone::Accent } else { Tone::Neutral };
        rsx! { Badge { tone, "{topic} → {queue}" } }
    });

    let env_info = session.env_info.as_deref().unwrap_or("");
    let sdk_ver = session.get_session_as_string().to_string();

    rsx! {
        div { class: "{row_class}",
            div { class: "msb-session__col",
                div { class: "msb-session__id selectable", "{session.id}" }
                {session_type}
            }
            div { class: "msb-session__col",
                div { class: "name selectable", "{session.name}" }
                div { class: "row",
                    span { class: "label", "SDK" }
                    span { class: "value", "{sdk_ver}" }
                }
                div { class: "row",
                    span { class: "label", "IP" }
                    span { class: "value", "{session.ip}" }
                    if !env_info.is_empty() {
                        Badge { tone: Tone::Accent, "{env_info}" }
                    }
                }
                div { class: "row",
                    span { class: "label", "Connected" }
                    span { class: "value", "{session.connected}" }
                }
                div { class: "row",
                    span { class: "label", "R" }
                    span { class: "value", "{r_size}" }
                    span { class: "label", "W" }
                    span { class: "value", "{w_size}" }
                }
                div { class: "row",
                    span { class: "label", "R/sec" }
                    span { class: "value", "{r_p_s}" }
                    span { class: "label", "W/sec" }
                    span { class: "value", "{w_p_s}" }
                }
            }
            div { class: "msb-session__col",
                div { class: "row",
                    span { class: "label", "Publishers" }
                }
                div { class: "msb-session__chips", {pub_chips} }
            }
            div { class: "msb-session__col",
                div { class: "row",
                    span { class: "label", "Subscribers" }
                }
                div { class: "msb-session__chips", {sub_chips} }
            }
        }
    }
}
