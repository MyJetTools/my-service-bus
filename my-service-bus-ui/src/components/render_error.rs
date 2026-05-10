use dioxus::prelude::*;

pub fn render_error(err: &str) -> Element {
    rsx! {
        div { class: "empty-state",
            h4 { style: "color:var(--red);", "Error" }
            p { style: "color:var(--fg-2); font-family:var(--font-mono); font-size:var(--fs-sm);",
                "{err}"
            }
        }
    }
}
