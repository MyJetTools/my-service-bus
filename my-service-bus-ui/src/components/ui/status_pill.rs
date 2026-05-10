use dioxus::prelude::*;

use super::Tone;

#[component]
pub fn StatusPill(tone: Tone, dot: Option<bool>, children: Element) -> Element {
    let tone_class = tone.class();
    let show_dot = dot.unwrap_or(false);
    rsx! {
        span { class: "msb-pill {tone_class}",
            if show_dot {
                span { class: "msb-pill__dot" }
            }
            {children}
        }
    }
}
