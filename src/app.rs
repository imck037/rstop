use crate::{
    ProcessCache, SortingMode, UiMode,
    system::{CpuState, CpuUsage},
};

pub struct Dashboard {
    pub cpus: Vec<CpuUsage>,
    pub stats: Vec<String>,
}

pub struct App {
    pub ui_mode: UiMode,
    pub sorting_mode: SortingMode,
    pub process_selected: usize,
    pub signal_selected: usize,
    pub proc_cache: ProcessCache,
    pub prev_cpus: Vec<CpuState>,
    pub core_count: usize,
    pub dashboard: Dashboard,
}
