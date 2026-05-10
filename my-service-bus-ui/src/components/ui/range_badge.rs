use dioxus::prelude::*;

const DIM_TAIL: usize = 5;

#[component]
pub fn MessageId(value: i64) -> Element {
    rsx! {
        span { class: "msb-mono", {render_value(value)} }
    }
}

#[component]
pub fn RangeBadge(from: i64, to: i64) -> Element {
    rsx! {
        span { class: "msb-badge is-neutral is-mono",
            {render_value(from)}
            "–"
            {render_value(to)}
        }
    }
}

fn render_value(value: i64) -> Element {
    let s = value.to_string();
    if s.len() <= DIM_TAIL {
        return rsx! { "{s}" };
    }
    let split = s.len() - DIM_TAIL;
    let head = s[..split].to_string();
    let tail = s[split..].to_string();
    rsx! {
        span { class: "dim", "{head}" }
        "{tail}"
    }
}
