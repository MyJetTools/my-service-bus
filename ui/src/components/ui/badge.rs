use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum Tone {
    Neutral,
    Success,
    Warning,
    Danger,
    Info,
    Accent,
    Purple,
    Pink,
}

impl Tone {
    pub fn class(self) -> &'static str {
        match self {
            Self::Neutral => "is-neutral",
            Self::Success => "is-success",
            Self::Warning => "is-warning",
            Self::Danger => "is-danger",
            Self::Info => "is-info",
            Self::Accent => "is-accent",
            Self::Purple => "is-purple",
            Self::Pink => "is-pink",
        }
    }
}

#[component]
pub fn Badge(tone: Tone, mono: Option<bool>, children: Element) -> Element {
    let tone_class = tone.class();
    let mono_class = if mono.unwrap_or(false) { " is-mono" } else { "" };
    rsx! {
        span { class: "msb-badge {tone_class}{mono_class}", {children} }
    }
}
