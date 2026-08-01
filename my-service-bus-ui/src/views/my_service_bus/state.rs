use std::collections::VecDeque;

use dioxus_utils::DataState;

use crate::models::{MySbHttpContract, NamespaceApiModel};

const KPI_HISTORY_CAP: usize = 60;

#[derive(Default)]
pub struct MySbState {
    pub started: bool,
    pub data: DataState<MySbHttpContract>,
    pub filter_string: String,
    pub active_section: SidebarSection,
    pub kpi_history: KpiHistory,
    pub last_updated_ms: f64,
    pub poll_failures: u32,
    /// Every namespace the node holds, refreshed by the polling loop.
    pub namespaces: Vec<NamespaceApiModel>,
    /// Namespace the UI is pointed at. Empty means the default one — see
    /// `crate::storage`.
    pub selected_namespace: String,
}

impl MySbState {
    /// Switching namespace throws away everything on screen: the topics, queues
    /// and KPI history all belong to the namespace we are leaving, and showing
    /// them next to the new namespace's name would be a lie until the next poll.
    pub fn switch_namespace(&mut self, namespace: String) {
        crate::storage::save_namespace(namespace.as_str());
        self.selected_namespace = namespace;
        self.data.reset();
        self.kpi_history.clear();
        self.last_updated_ms = 0.0;
    }

    pub fn push_kpi_sample(&mut self, data: &MySbHttpContract) {
        let bar = data.get_status_bar_calculated_values();
        self.kpi_history.push(KpiSample {
            msg_per_sec: bar.msg_per_sec as i32,
            persist_queue: bar.persist_queue as i32,
            incoming_kb_per_sec: (bar.incoming_per_sec / 1024) as i32,
            outgoing_kb_per_sec: (bar.outgoing_per_sec / 1024) as i32,
        });
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum SidebarSection {
    #[default]
    Topics,
    Sessions,
    Pages,
}

pub struct KpiSample {
    pub msg_per_sec: i32,
    pub persist_queue: i32,
    pub incoming_kb_per_sec: i32,
    pub outgoing_kb_per_sec: i32,
}

#[derive(Default)]
pub struct KpiHistory {
    pub msg_per_sec: VecDeque<i32>,
    pub persist_queue: VecDeque<i32>,
    pub incoming_kb_per_sec: VecDeque<i32>,
    pub outgoing_kb_per_sec: VecDeque<i32>,
}

impl KpiHistory {
    pub fn push(&mut self, sample: KpiSample) {
        push_capped(&mut self.msg_per_sec, sample.msg_per_sec);
        push_capped(&mut self.persist_queue, sample.persist_queue);
        push_capped(&mut self.incoming_kb_per_sec, sample.incoming_kb_per_sec);
        push_capped(&mut self.outgoing_kb_per_sec, sample.outgoing_kb_per_sec);
    }

    pub fn clear(&mut self) {
        self.msg_per_sec.clear();
        self.persist_queue.clear();
        self.incoming_kb_per_sec.clear();
        self.outgoing_kb_per_sec.clear();
    }
}

fn push_capped(buf: &mut VecDeque<i32>, value: i32) {
    if buf.len() >= KPI_HISTORY_CAP {
        buf.pop_front();
    }
    buf.push_back(value);
}
