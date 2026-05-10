use dioxus::prelude::*;

use crate::models::*;
pub fn render_queues(intervals: &[QueueIntervalModel]) -> Element {
    let q_periods = if intervals.len() == 0 {
        rsx! {
            div {}
        }
    } else {
        let mut result = Vec::new();
        for itm in intervals.iter() {
            if result.len() > 0 {
                result.push(rsx! { " " });
            }

            let from_message_id = super::render_message_id(itm.from_id);
            let to_message_id = super::render_message_id(itm.to_id);

            result.push(rsx! {
                {from_message_id}
                "-"
                {to_message_id}
            });
        }

        rsx! {
            span { class: "badge text-bg-success", {result.into_iter()} }
        }
    };

    q_periods
}
