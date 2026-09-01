use crate::signal::{SIGNALS, send_signal};
use crate::{SortingMode, UiMode, app::App, proc::Process};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_events(key: KeyEvent, app: &mut App, processes: &[Process]) -> bool {
    match app.ui_mode {
        UiMode::Normal => match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if app.process_selected < processes.len().saturating_sub(1) {
                    app.process_selected += 1;
                    true
                } else {
                    false
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.process_selected > 0 {
                    app.process_selected -= 1;
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('c') => {
                let changed = app.sorting_mode != SortingMode::Cpu;
                app.sorting_mode = SortingMode::Cpu;
                changed
            }
            KeyCode::Char('m') => {
                let changed = app.sorting_mode != SortingMode::Memory;
                app.sorting_mode = SortingMode::Memory;
                changed
            }
            KeyCode::Enter => {
                app.ui_mode = UiMode::SignalMenu;
                app.signal_selected = 0;
                true
            }
            _ => false,
        },
        UiMode::SignalMenu => match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if app.signal_selected < processes.len().saturating_sub(1) {
                    app.signal_selected += 1;
                    true
                } else {
                    false
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.signal_selected > 0 {
                    app.signal_selected -= 1;
                    true
                } else {
                    false
                }
            }
            KeyCode::Enter => {
                if let Some(p) = processes.get(app.process_selected) {
                    let sig = SIGNALS[app.signal_selected].value;
                    send_signal(p.pid.try_into().unwrap(), sig);
                }
                app.ui_mode = UiMode::Normal;
                true
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui_mode = UiMode::Normal;
                true
            }
            _ => false,
        },
    }
}
