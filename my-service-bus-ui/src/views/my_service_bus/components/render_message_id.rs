use dioxus::prelude::*;
pub fn render_message_id(message_id: i64) -> Element {
    let as_str = message_id.to_string();

    if as_str.len() < 5 {
        return rsx! {
            {as_str}
        };
    }

    rsx! {
        span { style: "color:gray", {&as_str[..as_str.len() - 5]} }
        {&as_str[as_str.len() - 5..]}
    }
}
