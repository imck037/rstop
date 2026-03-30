use crate::{
    SortingMode, UiMode,
    app::App,
    proc::{self, Process},
    signal, system, task,
};

use ratatui::layout::Rect;
use ratatui::widgets::Clear;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table, TableState},
};

pub fn render_ui(frame: &mut Frame, app: &mut App, processes: &mut Vec<Process>) {
    let area = frame.area();
    let curr_cpus = system::get_cpu_stat();
    let cpus = system::get_cpu_usage(&app.prev_cpus, &curr_cpus);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max((app.core_count) as u16),
            Constraint::Min(10),
        ])
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
        let bar_layout =
            Layout::horizontal([Constraint::Length(10), Constraint::Min(20)]).split(cpu_layout[i]);
        let gauge = Gauge::default()
            .style(Style::new().bg(Color::Reset).fg(Color::DarkGray))
            .block(Block::default().borders(Borders::empty()))
            .percent(usage.round() as u16)
            .label(format!("{:.1}%", usage));
        frame.render_widget(name, bar_layout[0]);
        frame.render_widget(gauge, bar_layout[1]);
    }

    app.prev_cpus = curr_cpus;

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

    match app.sorting_mode {
        SortingMode::Memory => {
            processes.sort_by(|a, b| b.memory.partial_cmp(&a.memory).unwrap());
        }
        SortingMode::Cpu => {
            processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        }
    }

    let visible_height = layout[1].height as usize - 3;

    let start = if app.process_selected >= visible_height {
        app.process_selected - visible_height + 1
    } else {
        0
    };

    let visible = processes.iter_mut().skip(start).take(visible_height);

    let rows: Vec<Row> = visible
        .map(|p| {
            proc::get_cpu_usage(p, app);
            Row::new(vec![
                p.pid.to_string(),
                p.name.to_string(),
                format!("{:.2}", p.cpu_usage),
                p.memory.to_string(),
                p.status.to_string(),
            ])
        })
        .collect();

    app.proc_cache.prev_total = proc::get_cpu_total_idle();
    let mut state = TableState::default();
    state.select(Some(app.process_selected - start));
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

    if let UiMode::SignalMenu = app.ui_mode {
        let signal_area = centered_layout(20, 20, area);

        let rows: Vec<Row> = signal::SIGNALS
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let style = if i == app.signal_selected {
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
