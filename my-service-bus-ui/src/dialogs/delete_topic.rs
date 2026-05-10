use dioxus::prelude::*;

use crate::components::ui::{BtnVariant, Button};

const DIALOG_TITLE: &str = "Delete topic";

#[derive(Default, Clone, Copy)]
struct State {
    hard_24h: bool,
}

#[component]
pub fn DeleteTopicDialog(topic_id: String, on_ok: EventHandler<String>) -> Element {
    let mut cs = use_signal(State::default);
    let hard_24h = cs.read().hard_24h;
    let topic_label = topic_id.clone();

    let content = rsx! {
        p {
            "Confirm to delete topic "
            b { "{topic_label}" }
            "?"
        }
        div { style: "display:flex; flex-direction: column; gap:6px; margin: 12px 0;",
            label { class: "msb-radio-label",
                input {
                    class: "msb-radio",
                    r#type: "radio",
                    name: "hard_delete_moment",
                    checked: !hard_24h,
                    onchange: move |_| cs.write().hard_24h = false,
                }
                "Now (immediate hard delete)"
            }
            label { class: "msb-radio-label",
                input {
                    class: "msb-radio",
                    r#type: "radio",
                    name: "hard_delete_moment",
                    checked: hard_24h,
                    onchange: move |_| cs.write().hard_24h = true,
                }
                "After 24 hours (allows restore)"
            }
        }
    };

    let ok_button = rsx! {
        Button {
            variant: BtnVariant::Danger,
            onclick: move |_| {
                let now_ms = js_sys::Date::now();
                let target_ms = if cs.read().hard_24h {
                    now_ms + 86_400_000.0
                } else {
                    now_ms
                };
                let date = js_sys::Date::new(&target_ms.into());
                let iso: String = date.to_iso_string().into();
                consume_context::<Signal<super::DialogState>>().set(super::DialogState::None);
                on_ok.call(iso);
            },
            "Delete"
        }
    };

    super::dialog_template(DIALOG_TITLE, content, ok_button)
}
