use crate::{ProcessCache, SortingMode, UiMode, system::CpuState};

pub struct App {
    pub ui_mode: UiMode,
    pub sorting_mode: SortingMode,
    pub process_selected: usize,
    pub signal_selected: usize,
    pub proc_cache: ProcessCache,
    pub prev_cpus: Vec<CpuState>,
    pub core_count: usize,
}
