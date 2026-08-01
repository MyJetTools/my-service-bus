use dioxus::prelude::*;

pub fn icon_search() -> Element {
    rsx! {
        svg {
            width: 14,
            height: 14,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: 2,
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: 11, cy: 11, r: 7 }
            line { x1: 21, y1: 21, x2: 16.65, y2: 16.65 }
        }
    }
}

pub fn icon_topics() -> Element {
    rsx! {
        svg {
            width: 16,
            height: 16,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: 1.8,
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M4 6h16M4 12h16M4 18h10" }
        }
    }
}

pub fn icon_sessions() -> Element {
    rsx! {
        svg {
            width: 16,
            height: 16,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: 1.8,
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M4 5h16v6H4zM4 13h16v6H4z" }
            circle { cx: 7, cy: 8, r: 1, fill: "currentColor", stroke: "none" }
            circle { cx: 7, cy: 16, r: 1, fill: "currentColor", stroke: "none" }
        }
    }
}

pub fn icon_pages() -> Element {
    rsx! {
        svg {
            width: 16,
            height: 16,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: 1.8,
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" }
            path { d: "M14 3v6h6" }
        }
    }
}

pub fn icon_plug() -> Element {
    rsx! {
        svg {
            width: 12,
            height: 12,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: 1.8,
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M9 2v6M15 2v6M5 8h14v3a7 7 0 0 1-14 0V8zM12 18v4" }
        }
    }
}
