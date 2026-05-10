use dioxus::prelude::*;

pub fn render_loading() -> Element {
    rsx! {
        div { class: "empty-state",
            style: "color:var(--fg-2); font-family:var(--font-mono); font-size:var(--fs-md);",
            "Loading…"
        }
    }
}
