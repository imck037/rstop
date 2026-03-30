use crate::signal::{SIGNALS, send_signal};
use crate::{SortingMode, UiMode, app::App, proc::Process};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_events(key: KeyEvent, app: &mut App, processes: Vec<Process>) {
    match app.ui_mode {
        UiMode::Normal => match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if app.process_selected < processes.len().saturating_sub(1) {
                    app.process_selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.process_selected > 0 {
                    app.process_selected -= 1;
                }
            }
            KeyCode::Char('c') => {
                app.sorting_mode = SortingMode::Cpu;
            }
            KeyCode::Char('m') => {
                app.sorting_mode = SortingMode::Memory;
            }
            KeyCode::Enter => {
                app.ui_mode = UiMode::SignalMenu;
                app.signal_selected = 0;
            }
            _ => {}
        },
        UiMode::SignalMenu => match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if app.signal_selected < processes.len().saturating_sub(1) {
                    app.signal_selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if app.signal_selected > 0 {
                    app.signal_selected -= 1;
                }
            }
            KeyCode::Enter => {
                if let Some(p) = processes.get(app.process_selected) {
                    let sig = SIGNALS[app.signal_selected].value;
                    send_signal(p.pid.try_into().unwrap(), sig);
                }
                app.ui_mode = UiMode::Normal;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui_mode = UiMode::Normal;
            }
            _ => {}
        },
    }
}
