use dioxus::prelude::*;

use crate::components::ui::RangeBadge;
use crate::models::*;

pub fn render_queues(intervals: &[QueueIntervalModel]) -> Element {
    if intervals.is_empty() {
        return rsx! {};
    }

    let items = intervals.iter().map(|itm| {
        rsx! {
            RangeBadge { from: itm.from_id, to: itm.to_id }
        }
    });

    rsx! { {items} }
}
