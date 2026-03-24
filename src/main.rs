mod proc;
mod system;
mod task;
mod test;
use std::collections::HashMap;
use std::{io, time::Duration};

use crossterm::{
    self,
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::widgets::Clear;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table, TableState},
};

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

fn send_signal(pid: i32, signal: i32) {
    unsafe {
        libc::kill(pid, signal);
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut prev_cpus = system::get_cpu_stat();

    let mut sorting_mode = SortingMode::Memory;
    let mut selected = 0usize;
    let mut signal_selected = 0usize;

    let mut proc_cache = ProcessCache {
        prev_proc: HashMap::new(),
        prev_total: proc::get_cpu_total_idle(),
    };

    let core_count = prev_cpus.len() - 1;

    let mut ui_mode = UiMode::Normal;

    const SIGNALS: &[SignalOption] = &[
        SignalOption {
            name: "SIGTERM",
            value: libc::SIGTERM,
        },
        SignalOption {
            name: "SIGKILL",
            value: libc::SIGKILL,
        },
        SignalOption {
            name: "SIGSTOP",
            value: libc::SIGSTOP,
        },
        SignalOption {
            name: "SIGCONT",
            value: libc::SIGCONT,
        },
    ];

    loop {
        let mut processes = proc::get_process();

        terminal.draw(|frame| {
            let area = frame.area();
            let curr_cpus = system::get_cpu_stat();
            let cpus = system::get_cpu_usage(&prev_cpus, &curr_cpus);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Max((core_count) as u16), Constraint::Min(10)])
                .split(area);

            let (total_memory, used_memory, ..) = system::get_memory();

            let mut total_swap = 0;
            let mut used_swap = 0;
            if let Some((total, used)) = system::get_swap() {
                total_swap = total / 1024;
                used_swap = used / 1024;
            }

            let uptime = system::get_uptime();
            let (total_task, running_task, .., total_threads) = task::tasks();

            let system_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50); 2])
                .split(layout[0]);

            let cpu_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(cpus.iter().skip(1).map(|_| Constraint::Length(1)))
                .split(system_layout[1]);

            for (i, cpu) in cpus.iter().skip(1).enumerate() {
                let usage = cpu.usage;
                let name = cpu.id.to_string();
                let bar_layout = Layout::horizontal([Constraint::Length(10), Constraint::Min(20)])
                    .split(cpu_layout[i]);
                let gauge = Gauge::default()
                    .style(Style::new().bg(Color::Reset).fg(Color::DarkGray))
                    .block(Block::default().borders(Borders::empty()))
                    .percent(usage.round() as u16)
                    .label(format!("{:.1}%", usage));
                frame.render_widget(name, bar_layout[0]);
                frame.render_widget(gauge, bar_layout[1]);
            }

            prev_cpus = curr_cpus;
            let curr_total = proc::get_cpu_total_idle();

            let stat: Vec<String> = vec![
                format!("Memory: {}MB/{}MB", used_memory / 1024, total_memory / 1024),
                format!("Swap: {}MB/{}MB", used_swap, total_swap),
                format!("Uptime: {}", uptime),
                format!(
                    "Tasks: {}, Running: {}, Threads: {}",
                    total_task, running_task, total_threads
                ),
            ];

            frame.render_widget(Paragraph::new(stat.join("\n")).centered(), system_layout[0]);

            match sorting_mode {
                SortingMode::Memory => {
                    processes.sort_by(|a, b| b.memory.partial_cmp(&a.memory).unwrap());
                }
                SortingMode::Cpu => {
                    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
                }
            }

            let visible_height = layout[1].height as usize - 3;

            let start = if selected >= visible_height {
                selected - visible_height + 1
            } else {
                0
            };

            let visible = processes.iter_mut().skip(start).take(visible_height);

            let rows: Vec<Row> = visible
                .map(|p| {
                    let prev = proc_cache.prev_proc.get(&p.pid).copied().unwrap_or(0);

                    let delta_proc = p.cpu_time.saturating_sub(prev);
                    let delta_total = curr_total.saturating_sub(proc_cache.prev_total);

                    if delta_total > 0 {
                        p.cpu_usage =
                            (delta_proc as f64 / delta_total as f64) * 100.0 * core_count as f64;
                    }
                    proc_cache.prev_proc.insert(p.pid, p.cpu_time);
                    Row::new(vec![
                        p.pid.to_string(),
                        p.name.to_string(),
                        format!("{:.2}", p.cpu_usage),
                        p.memory.to_string(),
                        p.status.to_string(),
                    ])
                })
                .collect();

            proc_cache.prev_total = curr_total;
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
                ],
            )
            .row_highlight_style(Style::default().bg(Color::DarkGray))
            .header(
                Row::new(vec!["PID", "Process", "CPU %", "Memory", "Status"]).style(
                    Style::default()
                        .black()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Gray),
                ),
            )
            .block(Block::default().title("Process").borders(Borders::all()));

            frame.render_stateful_widget(table, layout[1], &mut state);

            if let UiMode::SignalMenu = ui_mode {
                let signal_area = centered_layout(20, 20, area);

                let rows: Vec<Row> = SIGNALS
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let style = if i == signal_selected {
                            Style::default().bg(Color::Gray)
                        } else {
                            Style::default()
                        };

                        Row::new(vec![s.name]).style(style)
                    })
                    .collect();

                let signal_table = Table::new(rows, [Constraint::Percentage(100)]).block(
                    Block::default()
                        .title("Kill Signal")
                        .borders(Borders::all()),
                );
                frame.render_widget(Clear, area);
                frame.render_widget(signal_table, signal_area);
            }
        })?;

        if event::poll(Duration::from_millis(1000))? {
            if let Event::Key(key) = event::read()? {
                match ui_mode {
                    UiMode::Normal => match key.code {
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
                        KeyCode::Enter => {
                            ui_mode = UiMode::SignalMenu;
                            signal_selected = 0;
                        }
                        KeyCode::Char('q') => {
                            break;
                        }
                        _ => {}
                    },
                    UiMode::SignalMenu => match key.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            if signal_selected < processes.len().saturating_sub(1) {
                                signal_selected += 1;
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if signal_selected > 0 {
                                signal_selected -= 1;
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(p) = processes.get(selected) {
                                let sig = SIGNALS[signal_selected].value;
                                send_signal(p.pid.try_into().unwrap(), sig);
                            }
                            ui_mode = UiMode::Normal;
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            ui_mode = UiMode::Normal;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn centered_layout(x: u16, y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - y) / 2),
            Constraint::Percentage(y),
            Constraint::Percentage((100 - y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - x) / 2),
            Constraint::Percentage(x),
            Constraint::Percentage((100 - x) / 2),
        ])
        .split(popup_layout[1])[1]
}
