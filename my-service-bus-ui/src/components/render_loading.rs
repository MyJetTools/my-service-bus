use dioxus::prelude::*;
pub fn render_loading() -> Element {
    rsx! {
        h3 { style: "text-align:center", "Loading..." }
    }
}
