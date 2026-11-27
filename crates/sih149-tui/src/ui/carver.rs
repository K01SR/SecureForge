use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Wrap},
    text::{Line, Span},
};
use crate::{app::App, theme::Theme};
use super::format_bytes;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main[0]);

    render_config(frame, top[0], app);
    render_scan_progress(frame, top[1], app);
    render_results_header(frame, main[1], app);
    render_results(frame, main[2], app);
}

fn render_config(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.carver;
    let cursor = c.cursor_field;

    fn field<'a>(label: &'a str, val: &'a str, active: bool) -> Line<'a> {
        let val_color = if active { Theme::CYAN } else { Theme::TEXT };
        let border = if active { "▌ " } else { "  " };
        Line::from(vec![
            Span::styled(border, Style::default().fg(Theme::CYAN)),
            Span::styled(format!("{:<10}", label), Style::default().fg(Theme::MUTED)),
            Span::styled(
                if val.is_empty() { "[empty — type here]" } else { val },
                Style::default().fg(if val.is_empty() { Theme::MUTED } else { val_color })
                    .add_modifier(if active { Modifier::BOLD } else { Modifier::empty() }),
            ),
        ])
    }

    let conf_color = match c.min_confidence {
        0..=49 => Theme::DANGER,
        50..=74 => Theme::WARNING,
        _ => Theme::SUCCESS,
    };

    let lines = vec![
        Line::raw(""),
        field("Source:", &c.source, cursor == 0),
        Line::raw(""),
        field("Output:", &c.output_dir, cursor == 1),
        Line::raw(""),
        Line::from(vec![
            Span::styled(if cursor == 2 { "▌ " } else { "  " }, Style::default().fg(Theme::CYAN)),
            Span::styled("Confidence:", Style::default().fg(Theme::MUTED)),
            Span::styled(
                format!(" ≥{}%", c.min_confidence),
                Style::default().fg(conf_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  [↑↓ adjust]", Style::default().fg(Theme::MUTED)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  [Tab] ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("Next field  ", Style::default().fg(Theme::MUTED)),
            Span::styled("[Enter] ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::styled(if c.scanning { "Scanning…" } else { "Start Scan" }, Style::default().fg(Theme::MUTED)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if c.scanning { Theme::WARNING } else { Theme::BORDER }))
        .title(Span::styled(" ⬡ Forensic Carver  [Tab] Fields ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_scan_progress(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.carver;
    let pct = (c.progress * 100.0) as u16;
    let status = if c.scanning {
        format!("Scanning…  {:.1}%", c.progress * 100.0)
    } else if pct == 100 {
        format!("Complete — {} file(s) recovered", c.found.len())
    } else {
        "Ready — configure and press Enter".to_string()
    };

    let gauge_color = if c.scanning { Theme::WARNING }
        else if pct == 100 { Theme::SUCCESS }
        else { Theme::MUTED };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(gauge_color))
            .title(Span::styled(" Scan Progress ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(Theme::SURFACE)))
        .gauge_style(Style::default().fg(gauge_color))
        .percent(pct)
        .label(format!("  {}", status));
    frame.render_widget(gauge, rows[0]);

    // Log tail
    let log_items: Vec<ListItem> = c.log.iter().rev().take(8).map(|l| {
        ListItem::new(Line::from(Span::styled(format!(" › {}", l), Style::default().fg(Theme::MUTED))))
    }).collect();
    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Scan Log ", Style::default().fg(Theme::TEXT_DIM)))
        .style(Style::default().bg(Theme::BG));
    let log_list = List::new(log_items).block(log_block);
    frame.render_widget(log_list, rows[1]);
}

fn render_results_header(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.carver;
    let by_type: std::collections::HashMap<&str, usize> = c.found.iter().fold(Default::default(), |mut m, f| {
        *m.entry(f.file_type.as_str()).or_insert(0) += 1;
        m
    });

    let mut spans = vec![Span::styled("  Found: ", Style::default().fg(Theme::MUTED))];
    let mut types: Vec<_> = by_type.iter().collect();
    types.sort();
    for (t, n) in &types {
        spans.push(Span::styled(format!("{} {} ", n, t), Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled("│ ", Style::default().fg(Theme::BORDER)));
    }
    if types.is_empty() {
        spans.push(Span::styled("No files carved yet", Style::default().fg(Theme::MUTED)));
    }

    let line = Line::from(spans);
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::BG));
    let para = Paragraph::new(line).block(block);
    frame.render_widget(para, area);
}

fn render_results(frame: &mut Frame, area: Rect, app: &App) {
    let c = &app.carver;
    if c.found.is_empty() {
        let para = Paragraph::new(vec![
            Line::raw(""),
            Line::from(Span::styled(
                "  No carved files yet. Enter a source path and press Enter to start.",
                Style::default().fg(Theme::MUTED),
            )),
        ])
        .style(Style::default().bg(Theme::BG));
        frame.render_widget(para, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(" File Path").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Size").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Conf%").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Entropy").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
    ]).height(1).bottom_margin(1);

    let rows: Vec<Row> = c.found.iter().enumerate().map(|(i, f)| {
        let selected = c.result_cursor == i;
        let conf_color = match f.confidence {
            0..=49 => Theme::DANGER,
            50..=74 => Theme::WARNING,
            _ => Theme::SUCCESS,
        };
        let ent_color = Theme::entropy_color(f.entropy);
        let (_, ent_level) = crate::app::entropy_label(f.entropy);
        let ent_bar = "█".repeat(ent_level as usize).to_string() + &"░".repeat(5 - ent_level as usize);

        Row::new(vec![
            Cell::from(format!(" {}", f.path.rsplit('/').next().unwrap_or(&f.path)))
                .style(Style::default().fg(if selected { Theme::CYAN } else { Theme::TEXT })),
            Cell::from(f.file_type.clone())
                .style(Style::default().fg(Theme::BLUE)),
            Cell::from(format_bytes(f.size_bytes))
                .style(Style::default().fg(Theme::TEXT_DIM)),
            Cell::from(format!("{}%", f.confidence))
                .style(Style::default().fg(conf_color).add_modifier(Modifier::BOLD)),
            Cell::from(format!("{:.1}b {}", f.entropy, ent_bar))
                .style(Style::default().fg(ent_color)),
        ])
        .style(Style::default().bg(if selected { Theme::SURFACE2 } else { Theme::BG }))
        .height(1)
    }).collect();

    let widths = [
        Constraint::Min(0),
        Constraint::Length(8),
        Constraint::Length(9),
        Constraint::Length(6),
        Constraint::Length(16),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(
                " Carved Files  [↑↓] Navigate ",
                Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Theme::BG)));
    frame.render_widget(table, area);
}
