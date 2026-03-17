use std::{io, time::Duration};

use crossterm::{
    self,
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
};
use sysinfo::System;

#[derive(PartialEq)]
enum SortingMode {
    Cpu,
    Memory,
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new_all();

    let mut sorting_mode = SortingMode::Memory;
    let mut selected = 0usize;
    loop {
        sys.refresh_all();
        let mut processes: Vec<_> = sys.processes().iter().collect();

        terminal.draw(|f| {
            let area = f.area();

            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Min(10)])
                .split(area);

            let cpu = sys.global_cpu_usage();
            let used_memory = sys.used_memory() / 1024 / 1024;
            let total_memory = sys.total_memory() / 1024 / 1024;

            let stat = Paragraph::new(format!(
                "CPU: {:.2}%\nMemory: {} MB/ {} MB",
                cpu, used_memory, total_memory
            ))
            .block(
                Block::default()
                    .title("System status")
                    .borders(Borders::all()),
            );

            f.render_widget(stat, chunk[0]);

            match sorting_mode {
                SortingMode::Memory => {
                    processes.sort_by(|a, b| b.1.memory().partial_cmp(&a.1.memory()).unwrap());
                }
                SortingMode::Cpu => {
                    processes
                        .sort_by(|a, b| b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap());
                }
            }

            let visible_height = chunk[1].height as usize - 3;

            let start = if selected >= visible_height {
                selected - visible_height + 1
            } else {
                0
            };

            let visible = processes.iter().skip(start).take(visible_height);

            let rows: Vec<Row> = visible
                .map(|(pid, process)| {
                    Row::new(vec![
                        pid.to_string(),
                        format!("{:?}", process.name()),
                        format!("{:.2}", process.cpu_usage()),
                        format!("{}", process.memory() / 1024 / 1024),
                    ])
                })
                .collect();

            let mut state = TableState::default();
            state.select(Some(selected - start));
            let table = Table::new(
                rows,
                [
                    Constraint::Length(7),
                    Constraint::Length(20),
                    Constraint::Length(10),
                    Constraint::Length(10),
                ],
            )
            .row_highlight_style(Style::default().bg(Color::DarkGray))
            .header(
                Row::new(vec!["PID", "Process", "CPU %", "Memory"]).style(
                    Style::default()
                        .black()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Gray),
                ),
            )
            .block(Block::default().title("Process").borders(Borders::all()));

            f.render_stateful_widget(table, chunk[1], &mut state);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < processes.len().saturating_sub(1) {
                            selected += 1;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Char('c') => {
                        sorting_mode = SortingMode::Cpu;
                    }
                    KeyCode::Char('m') => {
                        sorting_mode = SortingMode::Memory;
                    }
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
