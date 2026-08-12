use dioxus::prelude::*;

use crate::components::ui::{
    format_micros, Badge, Led, LedColor, RangeBadge, Sparkline, SparklineColor, SparklineKind, Tone,
};
use crate::models::*;

/// A subscriber can hold thousands of ids on delivery; only the first ranges are
/// worth a badge, the rest is collapsed into a "+N" counter.
const MAX_RANGES_TO_RENDER: usize = 8;

pub fn render_subscriber(data: &MySbHttpContract, subscriber: &TopicSubscriber) -> Element {
    let is_active = subscriber.active != 0;
    let is_error = subscriber.delivery_state != 0;

    let row_class = if is_error { "msb-sub is-error" } else { "msb-sub" };
    let led_color = if is_active { LedColor::Blue } else { LedColor::Gray };
    let led_label = if is_active { "subscribed" } else { "idle" };

    let session = data.get_session(subscriber.session_id);
    let session_id_short = format!("{:02}", subscriber.session_id.unsigned_abs() % 100);

    let max_latency = subscriber.history.iter().map(|v| v.unsigned_abs() as i32).max().unwrap_or(0);
    let spark_color = SparklineColor::from_latency_micros(max_latency);
    let latency_text = format_micros(max_latency as i64);
    let history = subscriber.history.clone();

    let session_view = match session {
        Some(s) => {
            let sdk_ver = s.get_session_as_string().to_string();
            let ip = s.ip.clone();
            let session_name = s.name.clone();
            rsx! {
                div { class: "msb-sub__id", "id {subscriber.id}" }
                div { class: "msb-sub__meta",
                    span { class: "selectable", style: "color:var(--fg-1)", "{session_name}" }
                    Badge { tone: Tone::Neutral, mono: true, "{sdk_ver}" }
                    if !ip.is_empty() {
                        span { class: "msb-conn__host-tag", "{ip}" }
                    }
                    if let Some(state_str) = subscriber.delivery_state_str.as_ref() {
                        Badge {
                            tone: if is_error { Tone::Danger } else { Tone::Accent },
                            "{state_str}"
                        }
                    }
                }
            }
        }
        None => rsx! {
            div { class: "msb-sub__id", "id {subscriber.id}" }
            div { class: "msb-sub__meta",
                Badge { tone: Tone::Danger, "Unknown session" }
            }
        },
    };

    let delivery_view = render_delivery(subscriber);

    rsx! {
        div { class: "{row_class}",
            div { class: "msb-sub__pill",
                span { "{session_id_short}" }
                Led { color: led_color, label: led_label.to_string() }
            }
            div { class: "msb-sub__body", {session_view} {delivery_view} }
            div { class: "msb-sub__right",
                Sparkline {
                    kind: SparklineKind::Line,
                    width: 110,
                    height: 26,
                    color: spark_color,
                    data: history,
                }
                span { class: "msb-sub__latency", "{latency_text}" }
            }
        }
    }
}

/// Message ids the subscriber currently holds. `on delivery` shrinks and
/// `confirmed` grows as intermediary confirmations arrive, so the two together
/// show the delivery being worked through.
fn render_delivery(subscriber: &TopicSubscriber) -> Element {
    if subscriber.on_delivery_amount == 0 && subscriber.confirmed_amount == 0 {
        return rsx! {};
    }

    let on_delivery_ranges = render_ranges(&subscriber.on_delivery);
    let confirmed_ranges = render_ranges(&subscriber.confirmed);

    rsx! {
        div { class: "msb-sub__delivery",
            if subscriber.on_delivery_amount > 0 {
                span { class: "msb-sub__delivery-group",
                    span { class: "msb-sub__delivery-label", "on delivery" }
                    Badge { tone: Tone::Warning, mono: true, "{subscriber.on_delivery_amount}" }
                    {on_delivery_ranges}
                }
            }
            if subscriber.confirmed_amount > 0 {
                span { class: "msb-sub__delivery-group",
                    span { class: "msb-sub__delivery-label", "confirmed" }
                    Badge { tone: Tone::Success, mono: true, "{subscriber.confirmed_amount}" }
                    {confirmed_ranges}
                }
            }
        }
    }
}

fn render_ranges(intervals: &[QueueIntervalModel]) -> Element {
    let items = intervals.iter().take(MAX_RANGES_TO_RENDER).map(|itm| {
        rsx! {
            RangeBadge { from: itm.from_id, to: itm.to_id }
        }
    });

    let rest = intervals.len().saturating_sub(MAX_RANGES_TO_RENDER);

    rsx! {
        {items}
        if rest > 0 {
            span { class: "msb-sub__delivery-more", "+{rest}" }
        }
    }
}
