use std::{
    io,
    time::{Duration, Instant},
};

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
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table, TableState},
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
    let mb = 1024 * 1024;
    let refresh_rate = Duration::from_millis(200);
    let last_refresh = Instant::now();
    loop {
        sys.refresh_all();
        let mut processes: Vec<_> = sys.processes().iter().collect();

        terminal.draw(|f| {
            let area = f.area();

            let cpus = sys.cpus();
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length((cpus.len()) as u16),
                    Constraint::Min(10),
                ])
                .split(area);

            let used_memory = sys.used_memory() / mb;
            let total_memory = sys.total_memory() / mb;

            let cpu_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints(cpus.iter().map(|_| Constraint::Length(1)))
                .split(chunk[1]);

            for (i, cpu) in cpus.iter().enumerate() {
                let usage = cpu.cpu_usage();
                let name = cpu.name();
                let bar_layout = Layout::horizontal([Constraint::Length(10), Constraint::Min(20)])
                    .split(cpu_chunk[i]);
                let gauge = Gauge::default()
                    .style(Style::new().bg(Color::Reset).fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::empty()))
                    .percent(usage.round() as u16)
                    .label(format!("{:.1}%", usage));
                f.render_widget(name, bar_layout[0]);
                f.render_widget(gauge, bar_layout[1]);
            }

            let stat = Paragraph::new(format!("Memory: {} MB/ {} MB", used_memory, total_memory))
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

            let visible_height = chunk[2].height as usize - 3;

            let start = if selected >= visible_height {
                selected - visible_height + 1
            } else {
                0
            };

            let visible = processes.iter().skip(start).take(visible_height);

            let rows: Vec<Row> = visible
                .map(|(pid, process)| {
                    let display_name = if !process.cmd().is_empty() {
                        let name: Vec<&str> = process
                            .cmd()
                            .iter()
                            .filter_map(|part| part.to_str())
                            .collect();
                        format!("{}", name.join(" "))
                    } else if process.exe().unwrap() == "" {
                        format!("{:?}", process.name())
                    } else {
                        format!("{:?}", process.exe())
                    };
                    Row::new(vec![
                        pid.to_string(),
                        format!("{}", display_name),
                        format!("{:.2}", process.cpu_usage()),
                        format!("{:.2}", process.virtual_memory() / mb),
                        format!("{}", process.memory() / mb),
                        format!("{}", process.status()),
                        format!("{}", process.disk_usage().total_read_bytes / mb),
                        format!("{}", process.disk_usage().total_written_bytes / mb),
                        format!("{:?}", process.start_time()),
                    ])
                })
                .collect();

            let mut state = TableState::default();
            state.select(Some(selected - start));
            let table = Table::new(
                rows,
                [
                    Constraint::Length(7),
                    Constraint::Length(60),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(10),
                ],
            )
            .row_highlight_style(Style::default().bg(Color::DarkGray))
            .header(
                Row::new(vec![
                    "PID",
                    "Process",
                    "CPU %",
                    "Virt Mem",
                    "Memory",
                    "Status",
                    "Disk Read",
                    "Disk Write",
                    "Start Time",
                ])
                .style(
                    Style::default()
                        .black()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Gray),
                ),
            )
            .block(Block::default().title("Process").borders(Borders::all()));

            f.render_stateful_widget(table, chunk[2], &mut state);
        })?;

        if event::poll(Duration::from_millis(0))? {
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

        if last_refresh.elapsed() >= refresh_rate {
            sys.refresh_all();
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
