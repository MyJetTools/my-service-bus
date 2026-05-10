use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum BtnVariant {
    Primary,
    Danger,
    Ghost,
    OutlineDanger,
    OutlineSuccess,
}

impl BtnVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => "is-primary",
            Self::Danger => "is-danger",
            Self::Ghost => "is-ghost",
            Self::OutlineDanger => "is-outline-danger",
            Self::OutlineSuccess => "is-outline-success",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
#[allow(dead_code)]
pub enum BtnSize {
    #[default]
    Md,
    Sm,
    Xs,
}

impl BtnSize {
    fn class(self) -> &'static str {
        match self {
            Self::Md => "",
            Self::Sm => " is-sm",
            Self::Xs => " is-xs",
        }
    }
}

#[component]
pub fn Button(
    variant: BtnVariant,
    size: Option<BtnSize>,
    onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let v = variant.class();
    let s = size.unwrap_or_default().class();
    rsx! {
        button {
            r#type: "button",
            class: "msb-btn {v}{s}",
            onclick: move |e| onclick.call(e),
            {children}
        }
    }
}
