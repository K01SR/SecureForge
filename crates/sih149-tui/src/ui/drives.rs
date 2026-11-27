use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    text::{Line, Span},
};
use crate::{app::App, theme::Theme};
use super::format_bytes;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);

    render_table(frame, chunks[0], app);
    render_detail(frame, chunks[1], app);
}

fn render_table(frame: &mut Frame, area: Rect, app: &App) {
    let header = Row::new(vec![
        Cell::from("  Drive").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Model").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Type").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Size").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        Cell::from("Entropy").style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
    ]).height(1).bottom_margin(1);

    let rows: Vec<Row> = app.drives.iter().enumerate().map(|(i, d)| {
        let selected = app.drive_cursor == i;
        let status_text = if d.is_system { "SYSTEM" } else { "  Safe" };
        let status_color = if d.is_system { Theme::DANGER } else { Theme::SUCCESS };
        let indicator = if selected { "▶ " } else { "  " };

        // Entropy summary
        let (ent_text, ent_color) = if d.entropy_loaded && !d.entropy_samples.is_empty() {
            let avg = d.entropy_samples.iter().sum::<f64>() / d.entropy_samples.len() as f64;
            let (label, level) = crate::app::entropy_label(avg);
            let color = Theme::entropy_color(avg);
            let _ = level;
            (format!("{:.1}b {}", avg, label.split('/').next().unwrap_or("")), color)
        } else {
            ("─ not loaded".to_string(), Theme::MUTED)
        };

        let row_bg = if selected { Theme::SURFACE2 } else { Theme::BG };
        let text_color = if d.is_system { Theme::WARNING } else { Theme::TEXT };
        let name_color = if selected { Theme::CYAN } else { text_color };

        Row::new(vec![
            Cell::from(format!("{}{}", indicator, d.name))
                .style(Style::default().fg(name_color).add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() })),
            Cell::from(d.model.chars().take(20).collect::<String>())
                .style(Style::default().fg(Theme::TEXT_DIM)),
            Cell::from(d.drive_type.clone())
                .style(Style::default().fg(Theme::BLUE)),
            Cell::from(format_bytes(d.size_bytes))
                .style(Style::default().fg(Theme::TEXT)),
            Cell::from(status_text)
                .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Cell::from(ent_text)
                .style(Style::default().fg(ent_color)),
        ]).style(Style::default().bg(row_bg)).height(1)
    }).collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(22),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Min(0),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(
                " ⬡ Block Devices  [↑↓] Navigate  [Enter] Select  [E] Entropy  [R] Refresh ",
                Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Theme::BG)));
    frame.render_widget(table, area);
}

fn render_detail(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    // Drive info panel
    let lines = if let Some(d) = app.drives.get(app.drive_cursor) {
        let sys_tag = if d.is_system {
            ("⚠ SYSTEM / BOOT DRIVE — PROTECTED", Theme::DANGER)
        } else {
            ("✓ Safe — eligible for sanitization", Theme::SUCCESS)
        };

        let avg_entropy = if d.entropy_loaded && !d.entropy_samples.is_empty() {
            let avg = d.entropy_samples.iter().sum::<f64>() / d.entropy_samples.len() as f64;
            let (label, _) = crate::app::entropy_label(avg);
            format!("{:.2} bits/byte  ({})", avg, label)
        } else {
            "Not analyzed yet — press E".to_string()
        };

        let size_str = format_bytes(d.size_bytes);

        vec![
            Line::raw(""),
            Line::from(Span::styled(format!("  /dev/{}", d.name), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Model:   ", Style::default().fg(Theme::MUTED)),
                Span::styled(d.model.clone(), Style::default().fg(Theme::TEXT)),
            ]),
            Line::from(vec![
                Span::styled("  Type:    ", Style::default().fg(Theme::MUTED)),
                Span::styled(d.drive_type.clone(), Style::default().fg(Theme::TEXT)),
            ]),
            Line::from(vec![
                Span::styled("  Size:    ", Style::default().fg(Theme::MUTED)),
                Span::styled(size_str, Style::default().fg(Theme::TEXT)),
            ]),
            Line::from(vec![
                Span::styled("  Path:    ", Style::default().fg(Theme::MUTED)),
                Span::styled(d.path.clone(), Style::default().fg(Theme::TEXT)),
            ]),
            Line::raw(""),
            Line::from(Span::styled(format!("  {}", sys_tag.0), Style::default().fg(sys_tag.1).add_modifier(Modifier::BOLD))),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  Entropy: ", Style::default().fg(Theme::MUTED)),
                Span::styled(avg_entropy, Style::default().fg(Theme::PURPLE)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled("  [Enter] ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                Span::styled("Select for Wipe  ", Style::default().fg(Theme::MUTED)),
                Span::styled("[E] ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)),
                Span::styled("Entropy View", Style::default().fg(Theme::MUTED)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled("  No drives found. Try running with sudo.", Style::default().fg(Theme::WARNING)))]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Drive Detail ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, rows[0]);

    // Mini entropy preview (if loaded)
    render_mini_entropy(frame, rows[1], app);
}

fn render_mini_entropy(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Entropy Preview  [E] Full View ", Style::default().fg(Theme::PURPLE)))
        .style(Style::default().bg(Theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(d) = app.drives.get(app.drive_cursor) {
        if d.entropy_loaded && !d.entropy_samples.is_empty() {
            let width = inner.width as usize;
            let samples = &d.entropy_samples;
            let step = (samples.len() as f64 / width as f64).max(1.0);

            let mut spans = vec![Span::raw(" ")];
            for i in 0..width.min(inner.width as usize) {
                let idx = (i as f64 * step) as usize;
                let e = samples.get(idx).copied().unwrap_or(0.0);
                let color = Theme::entropy_color(e);
                let ch = Theme::entropy_bar_char(e);
                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }

            let line = Line::from(spans);
            let para = Paragraph::new(vec![Line::raw(""), line]);
            frame.render_widget(para, inner);
        } else {
            let para = Paragraph::new("  Press E to load entropy analysis")
                .style(Style::default().fg(Theme::MUTED));
            frame.render_widget(para, inner);
        }
    }
}

fn kv<'a>(label: &'a str, val: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Theme::MUTED)),
        Span::styled(val, Style::default().fg(Theme::TEXT)),
    ])
}
