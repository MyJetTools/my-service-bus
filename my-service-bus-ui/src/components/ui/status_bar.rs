use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum StatusValueTone {
    Default,
    Success,
    Warning,
    Danger,
    Dim,
}

impl StatusValueTone {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Success => " is-success",
            Self::Warning => " is-warning",
            Self::Danger => " is-danger",
            Self::Dim => " is-dim",
        }
    }
}

pub fn status_item(label: &str, value: &str, tone: StatusValueTone) -> Element {
    let label = label.to_string();
    let value = value.to_string();
    let value_class = tone.class();
    rsx! {
        div { class: "msb-statusbar__item",
            span { class: "msb-statusbar__label", "{label}" }
            span { class: "msb-statusbar__value{value_class}", "{value}" }
        }
    }
}

pub fn status_item_end(label: &str, value: &str, tone: StatusValueTone) -> Element {
    let label = label.to_string();
    let value = value.to_string();
    let value_class = tone.class();
    rsx! {
        div { class: "msb-statusbar__item is-end",
            span { class: "msb-statusbar__label", "{label}" }
            span { class: "msb-statusbar__value{value_class}", "{value}" }
        }
    }
}

pub fn status_live(is_live: bool) -> Element {
    let class = if is_live {
        "msb-statusbar__live"
    } else {
        "msb-statusbar__live is-stale"
    };
    rsx! {
        div { class: "msb-statusbar__item",
            span { class: "{class}",
                span { class: "msb-statusbar__live-dot" }
                if is_live { "Live" } else { "Stale" }
            }
        }
    }
}
