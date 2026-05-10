use std::collections::VecDeque;

use dioxus_utils::DataState;

use crate::models::MySbHttpContract;

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
}

impl MySbState {
    pub fn push_kpi_sample(&mut self, data: &MySbHttpContract) {
        let bar = data.get_status_bar_calculated_values();
        let mem_pct = if data.system.totalmem > 0 {
            ((data.system.usedmem as f64 / data.system.totalmem as f64) * 100.0) as i32
        } else {
            0
        };
        self.kpi_history.push(KpiSample {
            msg_per_sec: bar.msg_per_sec as i32,
            persist_queue: bar.persist_queue as i32,
            sessions: data.sessions.items.len() as i32,
            mem_used_pct: mem_pct,
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
    pub sessions: i32,
    pub mem_used_pct: i32,
}

#[derive(Default)]
pub struct KpiHistory {
    pub msg_per_sec: VecDeque<i32>,
    pub persist_queue: VecDeque<i32>,
    pub sessions: VecDeque<i32>,
    pub mem_used_pct: VecDeque<i32>,
}

impl KpiHistory {
    pub fn push(&mut self, sample: KpiSample) {
        push_capped(&mut self.msg_per_sec, sample.msg_per_sec);
        push_capped(&mut self.persist_queue, sample.persist_queue);
        push_capped(&mut self.sessions, sample.sessions);
        push_capped(&mut self.mem_used_pct, sample.mem_used_pct);
    }

    pub fn clear(&mut self) {
        self.msg_per_sec.clear();
        self.persist_queue.clear();
        self.sessions.clear();
        self.mem_used_pct.clear();
    }
}

fn push_capped(buf: &mut VecDeque<i32>, value: i32) {
    if buf.len() >= KPI_HISTORY_CAP {
        buf.pop_front();
    }
    buf.push_back(value);
}
