use dioxus::prelude::*;

use crate::components::ui::{Badge, Led, LedColor, Tone};
use crate::models::*;

pub fn render_topic_connections<'a>(
    data: &'a MySbHttpContract,
    topic: &'a TopicHttpModel,
) -> Element {
    if topic.publishers.is_empty() {
        return rsx! {
            div { class: "empty-state",
                style: "margin:20px auto; text-align:center;",
                h4 { "No publishers" }
                p { "No client is currently publishing into this topic." }
            }
        };
    }

    let items = topic.publishers.iter().map(|publisher| {
        let is_live = publisher.active != 0;
        let conn_class = if is_live { "msb-conn is-live" } else { "msb-conn" };
        let session = data.get_session(publisher.session_id);

        let session_id_short = format!("{:02}", publisher.session_id.unsigned_abs() % 100);

        let (name, ip, version, env_info) = match session {
            Some(s) => (
                s.name.clone(),
                s.ip.clone(),
                Some(s.get_session_as_string().to_string()),
                s.env_info.clone(),
            ),
            None => ("Session not found".to_string(), String::new(), None, None),
        };

        let led_color = if is_live { LedColor::Green } else { LedColor::Gray };
        let led_label = if is_live { "publishing" } else { "idle" };

        rsx! {
            div { class: "{conn_class}",
                div { class: "msb-conn__pill",
                    span { "{session_id_short}" }
                    Led { color: led_color, label: led_label.to_string() }
                }
                div { class: "msb-conn__body",
                    div { class: "msb-conn__name",
                        span { class: "selectable", "{name}" }
                        if let Some(ver) = version.as_ref() {
                            span { class: "msb-conn__ver", "{ver}" }
                        }
                    }
                    div { class: "msb-conn__meta",
                        if !ip.is_empty() {
                            span { class: "msb-conn__host-tag", "{ip}" }
                        }
                        if let Some(env) = env_info.as_ref() {
                            Badge { tone: Tone::Accent, "{env}" }
                        }
                        if is_live {
                            Badge { tone: Tone::Success,
                                span { class: "msb-pill__dot",
                                    style: "width:5px;height:5px;background:currentColor;border-radius:50%;display:inline-block;margin-right:2px;"
                                }
                                "publishing"
                            }
                        }
                    }
                }
                div { class: "msb-conn__right",
                    Badge { tone: Tone::Neutral, mono: true, "id {publisher.session_id}" }
                }
            }
        }
    });

    rsx! { {items} }
}
