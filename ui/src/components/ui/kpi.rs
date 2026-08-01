use std::collections::VecDeque;

use dioxus::prelude::*;

use super::{Sparkline, SparklineColor, SparklineKind};

#[derive(Clone, Copy, PartialEq)]
pub enum KpiTone {
    Default,
    Warning,
    Danger,
}

impl KpiTone {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Warning => " is-warning",
            Self::Danger => " is-danger",
        }
    }
}

#[component]
pub fn KpiCard(
    label: String,
    value: String,
    unit: Option<String>,
    tone: KpiTone,
    color: SparklineColor,
    history: Vec<i32>,
) -> Element {
    let value_class = tone.class();
    let unit_view = unit.unwrap_or_default();
    rsx! {
        div { class: "msb-kpi",
            div { class: "msb-kpi__label", "{label}" }
            div { class: "msb-kpi__value{value_class}",
                "{value}"
                if !unit_view.is_empty() {
                    span { class: "unit", "{unit_view}" }
                }
            }
            div { class: "msb-kpi__spark",
                Sparkline {
                    kind: SparklineKind::Area,
                    width: 80,
                    height: 28,
                    color,
                    data: history,
                }
            }
        }
    }
}

pub fn vec_from(buf: &VecDeque<i32>) -> Vec<i32> {
    buf.iter().copied().collect()
}
