use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum SparklineKind {
    Area,
    Line,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SparklineColor {
    Accent,
    Green,
    Amber,
    Red,
}

impl SparklineColor {
    fn stroke(self) -> &'static str {
        match self {
            Self::Accent => "#7c9eff",
            Self::Green => "#4ade80",
            Self::Amber => "#fbbf24",
            Self::Red => "#f87171",
        }
    }

    pub fn from_latency_micros(micros: i32) -> Self {
        let m = micros.unsigned_abs();
        if m < 1_000 {
            Self::Green
        } else if m < 10_000 {
            Self::Accent
        } else if m < 50_000 {
            Self::Amber
        } else {
            Self::Red
        }
    }
}

static GRAD_SEQ: AtomicU64 = AtomicU64::new(0);

#[component]
pub fn Sparkline(
    kind: SparklineKind,
    width: i32,
    height: i32,
    color: SparklineColor,
    data: Vec<i32>,
) -> Element {
    let uid = use_hook(|| GRAD_SEQ.fetch_add(1, Ordering::Relaxed));
    let grad_id = format!("msb-spark-grad-{uid}");

    if data.is_empty() {
        return render_empty(width, height);
    }

    let max = data.iter().map(|v| v.unsigned_abs() as i32).max().unwrap_or(0);
    if max == 0 {
        return render_empty(width, height);
    }

    let n = data.len() as f64;
    let dx = if n > 1.0 { width as f64 / (n - 1.0) } else { width as f64 };
    let h = height as f64;
    let max_f = max as f64;

    let mut path = String::with_capacity(data.len() * 14);
    for (i, v) in data.iter().enumerate() {
        let x = i as f64 * dx;
        let y = h - (v.unsigned_abs() as f64 / max_f) * h;
        if i == 0 {
            path.push_str(&format!("M {x:.2} {y:.2}"));
        } else {
            path.push_str(&format!(" L {x:.2} {y:.2}"));
        }
    }

    let stroke = color.stroke();

    match kind {
        SparklineKind::Line => rsx! {
            svg {
                class: "msb-sparkline",
                width: "{width}",
                height: "{height}",
                view_box: "0 0 {width} {height}",
                preserve_aspect_ratio: "none",
                role: "img",
                path {
                    d: "{path}",
                    fill: "none",
                    stroke: "{stroke}",
                    stroke_width: 1.4,
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
            }
        },
        SparklineKind::Area => {
            let last_x = (n - 1.0).max(0.0) * dx;
            let area_path = format!("{path} L {last_x:.2} {h:.2} L 0 {h:.2} Z");
            rsx! {
                svg {
                    class: "msb-sparkline",
                    width: "{width}",
                    height: "{height}",
                    view_box: "0 0 {width} {height}",
                    preserve_aspect_ratio: "none",
                    role: "img",
                    defs {
                        linearGradient {
                            id: "{grad_id}",
                            x1: 0,
                            y1: 0,
                            x2: 0,
                            y2: 1,
                            stop { offset: "0%", "stop-color": "{stroke}", "stop-opacity": 0.55 }
                            stop { offset: "100%", "stop-color": "{stroke}", "stop-opacity": 0.0 }
                        }
                    }
                    path { d: "{area_path}", fill: "url(#{grad_id})" }
                    path {
                        d: "{path}",
                        fill: "none",
                        stroke: "{stroke}",
                        stroke_width: 1.2,
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                    }
                }
            }
        }
    }
}

fn render_empty(width: i32, height: i32) -> Element {
    let mid = height as f64 / 2.0;
    rsx! {
        svg {
            class: "msb-sparkline",
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            preserve_aspect_ratio: "none",
            role: "img",
            "aria-label": "no data",
            line {
                x1: 0,
                y1: "{mid:.2}",
                x2: "{width}",
                y2: "{mid:.2}",
                stroke: "rgba(248,113,113,0.45)",
                stroke_width: 1,
                stroke_dasharray: "2 3",
            }
        }
    }
}

pub fn format_micros(micros: i64) -> String {
    let m = micros.unsigned_abs();
    if m < 1_000 {
        format!("{m}µs")
    } else if m < 1_000_000 {
        format!("{:.1}ms", m as f64 / 1_000.0)
    } else {
        format!("{:.2}s", m as f64 / 1_000_000.0)
    }
}
