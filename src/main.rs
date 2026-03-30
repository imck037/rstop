mod app;
mod events;
mod proc;
mod signal;
mod system;
mod task;
mod test;
mod ui;
use crate::ui::render_ui;
use std::collections::HashMap;
use std::io;

use crossterm::{
    self,
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::App;

#[derive(PartialEq)]
enum SortingMode {
    Cpu,
    Memory,
}

enum UiMode {
    Normal,
    SignalMenu,
}

struct ProcessCache {
    prev_proc: HashMap<usize, usize>,
    prev_total: usize,
}

struct SignalOption {
    name: &'static str,
    value: i32,
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let prev_cpus = system::get_cpu_stat();
    let core_count = prev_cpus.len() - 1;

    let proc_cache = ProcessCache {
        prev_proc: HashMap::new(),
        prev_total: proc::get_cpu_total_idle(),
    };

    let mut app = App {
        sorting_mode: SortingMode::Memory,
        signal_selected: 0,
        process_selected: 0,
        ui_mode: UiMode::Normal,
        core_count: core_count,
        prev_cpus: prev_cpus,
        proc_cache: proc_cache,
    };

    loop {
        let mut processes = proc::get_process();

        terminal.draw(|frame| {
            render_ui(frame, &mut app, &mut processes);
        })?;

        if let Event::Key(key) = event::read()? {
            if let UiMode::Normal = app.ui_mode {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
            events::handle_events(key, &mut app, processes);
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
