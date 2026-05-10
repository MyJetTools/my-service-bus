use dioxus::prelude::*;

pub fn dialog_template(title: &str, content: Element, btn_success: Element) -> Element {
    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-card",
                h3 { style: "margin-top:0", {title} }
                div { {content} }
                div { style: "display:flex; justify-content:flex-end; gap:10px; margin-top:16px",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            consume_context::<Signal<super::DialogState>>().set(super::DialogState::None)
                        },
                        "Cancel"
                    }
                    {btn_success}
                }
            }
        }
    }
}
