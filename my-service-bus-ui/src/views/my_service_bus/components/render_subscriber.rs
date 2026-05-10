use dioxus::prelude::*;

use crate::models::*;

pub fn render_subscriber(data: &MySbHttpContract, subscriber: &TopicSubscriber) -> Element {
    let subscriber_led = if subscriber.active == 0 {
        rsx! {
            div { class: "led-gray" }
        }
    } else {
        rsx! {
            div { class: "led-blue" }
        }
    };

    let subscriber_style = if subscriber.delivery_state == 0 {
        "text-bg-primary"
    } else {
        "text-bg-danger"
    };

    let subscriber_info = if let Some(str) = subscriber.delivery_state_str.as_ref() {
        rsx! {
            div { style: "text-align:right; padding: 0",
                span { class: "badge {subscriber_style}", {str.as_str()} }
            }
        }
    } else {
        rsx! {}
    };

    let session = data.get_session(subscriber.session_id);

    let session_to_render = match session {
        Some(session) => {
            let graph = super::render_graph(false, &subscriber.history);
            rsx! {
                div { class: "info-line-xs",
                    b { "MY-SB-SDK ver: " }
                    "{session.get_session_as_string()}"
                }
                div { class: "info-line-xs", "{session.ip}" }
                div { {graph} }
            }
        }
        None => {
            rsx! {
                div { "Unknown session" }
            }
        }
    };

    rsx! {
        div { class: "topic-subscriber",
            div {
                {subscriber_led}
                div {
                    span { class: "badge text-bg-dark", {subscriber.session_id.to_string()} }
                }

                div {
                    span { class: "badge {subscriber_style}", {subscriber.id.to_string()} }
                }
            }
            div { style: "text-align:left",
                {subscriber_info}
                {session_to_render}
            }
        }

    }
}
