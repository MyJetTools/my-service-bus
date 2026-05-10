use dioxus::prelude::*;

pub fn render_error(err: &str) -> Element {
    rsx! {
        div { class: "alert alert-danger", {err} }
    }
}
