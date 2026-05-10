use dioxus::prelude::*;


use crate::{models::*, utils::format_mem};

pub fn render_sessions(data: &MySbHttpContract,  filter_string: &str) -> Element{

    let mut odd = false;

    let sessions_to_render = data.sessions.items.iter()
    .filter(|session|session.filter_me(filter_string))
    . map(|session| {
        let bg_color = if odd {
            "--vz-table-active-bg"
        } else {
            "--vz-table-striped-bg"
        };

        odd = !odd;

        let session_type = match session.get_session_type() {
            SessionType::Tcp => {
                rsx! {
                    span { class: "badge text-bg-success", "Tcp" }
                }
            }
            SessionType::Http => {
                rsx! {
                    span {
                        span { class: "badge text-bg-warning", "Http" }
                    }
                }
            }
        };

        let r_size = format_mem(session.read_size);
        let w_size = format_mem(session.written_size);

        let r_p_s = format_mem(session.read_per_sec);
        let w_p_s = format_mem(session.written_per_sec);


        
        let (publishers, subscribers) =  data.get_publishers_and_subscribers(session.id);


        let publishers_to_render = publishers.into_iter().map(|(publisher, active)|{


            if active>0{
                rsx! {
                    span { class: "badge text-bg-success my-badge", {publisher} }
                }
    
            }else {
                rsx! {
                    span { class: "badge text-bg-light my-badge", {publisher} }
                }
                    
            }

        });


        let subscribers_to_render = subscribers.into_iter().map(|(topic, queue, active)|{
            if active>0{
                rsx! {
                    span { class: "badge text-bg-success my-badge", "{topic}->{queue}" }
                }
    
            }else {
                rsx! {
                    span { class: "badge text-bg-light my-badge", "{topic}->{queue}" }
                }
                    
            }
        });

        
        let env_info = if let Some(env_info) = session.env_info.as_ref() {
            rsx! {
                span {
                    style: "background: white;color: black;",
                    class: "badge badge-light",
                    "{env_info.as_str()}"
                }
            }
        } else {
            rsx! {}
        };

        rsx! {
            tr { style: "--bg-color:var({bg_color}); background-color:var({bg_color}); vertical-align: top;border-bottom: 1px solid black;",
                td {
                    div { class: "info-line", "{session.id}" }
                    div { class: "info-line", {session_type} }
                }
                td {
                    div { class: "info-line-bold", "{session.name}" }

                    div { class: "info-line-xs",
                        b { "MY-SB-SDK ver: " }
                        "{session.get_session_as_string()}"
                    }

                    div { class: "info-line-xs",
                        b { "Ip: " }
                        "{session.ip} "
                        {env_info}
                    }

                    div { class: "info-line-xs",
                        b { "Connected: " }
                        "{session.connected}"
                    }
                    div { class: "info-line-xs",
                        b { "Read: " }
                        {r_size}
                    }
                    div { class: "info-line-xs",
                        b { "Written: " }
                        {w_size}
                    }
                    div { class: "info-line-xs",
                        b { "R/sec: " }
                        {r_p_s}
                    }
                    div { class: "info-line-xs",
                        b { "W/sec: " }
                        {w_p_s}
                    }
                }
                td { {publishers_to_render} }
                td { {subscribers_to_render} }
            }
        }
    });


    rsx!{
        {sessions_to_render}
    }
    
}
