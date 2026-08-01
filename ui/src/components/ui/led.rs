use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum LedColor {
    Gray,
    Green,
    Blue,
    Amber,
    Red,
}

impl LedColor {
    fn class(self) -> &'static str {
        match self {
            Self::Gray => "is-gray",
            Self::Green => "is-green",
            Self::Blue => "is-blue",
            Self::Amber => "is-amber",
            Self::Red => "is-red",
        }
    }
}

#[component]
pub fn Led(color: LedColor, label: Option<String>) -> Element {
    let c = color.class();
    let aria = label.unwrap_or_default();
    rsx! {
        span {
            class: "msb-led {c}",
            "aria-label": "{aria}",
            role: "img",
        }
    }
}
