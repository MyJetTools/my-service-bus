use dioxus::prelude::*;

const WIDTH: i64 = 200;
const HEIGHT: i64 = 16;

pub fn render_page(page_no: i64, amount: i64, size: i64, sub_pages: &[i32]) -> Element {
    let bars = sub_pages.iter().map(|p| {
        let p = *p as i64;
        let x = p * 2;
        rsx! {
            line {
                x1: x,
                y1: 0,
                x2: x,
                y2: HEIGHT,
                stroke: "var(--accent)",
                stroke_width: 1,
            }
        }
    });

    rsx! {
        div { class: "msb-page",
            div { class: "msb-page__head",
                span { class: "label", "Page" }
                span { class: "value", "{page_no}" }
                span { class: "label", "Amount" }
                span { class: "value", "{amount}" }
                span { class: "label", "Size" }
                span { class: "value", "{size}" }
            }
            svg {
                width: "{WIDTH}",
                height: "{HEIGHT}",
                view_box: "0 0 {WIDTH} {HEIGHT}",
                preserve_aspect_ratio: "none",
                rect {
                    width: "{WIDTH}",
                    height: "{HEIGHT}",
                    rx: 3,
                    ry: 3,
                    fill: "var(--bg-3)",
                    stroke: "var(--border)",
                    stroke_width: 1,
                }
                {bars}
            }
        }
    }
}
