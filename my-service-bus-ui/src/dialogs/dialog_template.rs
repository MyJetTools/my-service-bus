use dioxus::prelude::*;

use crate::components::ui::{BtnVariant, Button};

pub fn dialog_template(title: &str, content: Element, btn_success: Element) -> Element {
    let title = title.to_string();
    rsx! {
        div {
            class: "msb-dialog__backdrop",
            onclick: move |_| {
                consume_context::<Signal<super::DialogState>>().set(super::DialogState::None);
            },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    consume_context::<Signal<super::DialogState>>().set(super::DialogState::None);
                }
            },
            div {
                class: "msb-dialog__card",
                onclick: move |e| { e.stop_propagation(); },
                div { class: "msb-dialog__header", "{title}" }
                div { class: "msb-dialog__body", {content} }
                div { class: "msb-dialog__footer",
                    Button {
                        variant: BtnVariant::Ghost,
                        onclick: move |_| {
                            consume_context::<Signal<super::DialogState>>().set(super::DialogState::None);
                        },
                        "Cancel"
                    }
                    {btn_success}
                }
            }
        }
    }
}
