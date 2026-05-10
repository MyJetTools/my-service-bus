use std::time::Duration;

use dioxus::prelude::*;

const WIDTH: i64 = 240;
const HEIGHT: i64 = 50;

pub fn render_graph(is_amount: bool, elements: &[i32]) -> Element {
    let mut x = 0;

    let height = HEIGHT as f64;

    let max = elements
        .into_iter()
        .map(|v| if *v > 0 { *v } else { -*v })
        .max();

    let items: Vec<_> = match max {
        Some(max) => {
            let max = if max == 0 { 1 } else { max };
            elements
                .into_iter()
                .map(|v| {
                    let v = *v;
                    let (v, color) = if v > 0 { (v, "darkgray") } else { (-v, "red") };

                    let v = v as f64;
                    let max = max as f64;

                    let y2 = height - v / max * height;

                    let the_x = x;

                    x += 2;
                    rsx! {
                        line {
                            x1: "{the_x}",
                            y1: "{HEIGHT}",
                            x2: "{the_x}",
                            y2: "{y2}",
                            style: "stroke:{color};stroke-width:2",
                        }
                    }
                })
                .collect()
        }
        None => vec![],
    };

    let duration_str;
    let duration_str2;

    let text = match max {
        Some(max) => {
            let mut result = Vec::with_capacity(2);

            if max == 0 {
                duration_str = "0".to_string();
            } else {
                if is_amount {
                    duration_str = max.to_string();
                } else {
                    let duration = Duration::from_micros(max as u64);
                    duration_str = format!("{:?}", duration);
                }
            }

            duration_str2 = duration_str.clone();

            result.push(rsx! {
                text { x: 1, y: 13, fill: "black", "{duration_str.as_str()}" }
            });

            result.push(rsx! {
                text { x: 0, y: 12, fill: "lime", "{duration_str2.as_str()}" }
            });

            result
        }
        None => vec![],
    };

    rsx! {
        svg {
            style: "font-size:12px; margin:0; padding:0;",
            width: WIDTH,
            height: HEIGHT,
            rect {
                style: "fill: var(--bg-color); stroke-width:1; stroke:black;",
                width: WIDTH,
                height: HEIGHT,
            }
            {items.into_iter()}
            {text.into_iter()}
        }
    }
}
