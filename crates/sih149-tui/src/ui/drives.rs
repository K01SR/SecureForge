use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell}, text::{Line, Span}};
use crate::{app::App, theme::Theme};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Drive table
    let header_cells = ["  Drive", "Model", "Type", "Size", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Theme::ACCENT).bold()));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows: Vec<Row> = app.drives.iter().enumerate().map(|(i, d)| {
        let is_selected = app.drive_cursor == i;
        let status_text = if d.is_system { "SYSTEM" } else { "Safe" };
        let status_color = if d.is_system { Theme::DANGER } else { Theme::SUCCESS };
        let row_style = if is_selected {
            Style::default().fg(Theme::BG).bg(Theme::ACCENT)
        } else if d.is_system {
            Style::default().fg(Theme::DANGER)
        } else {
            Style::default().fg(Theme::TEXT)
        };

        let indicator = if is_selected { "▶ " } else { "  " };
        let cells = vec![
            Cell::from(format!("{}{}", indicator, d.name)),
            Cell::from(d.model.chars().take(18).collect::<String>()),
            Cell::from(d.drive_type.clone()),
            Cell::from(format_bytes(d.size_bytes)),
            Cell::from(status_text).style(Style::default().fg(status_color)),
        ];
        Row::new(cells).style(row_style).height(1)
    }).collect();

    let table = Table::new(rows, [Constraint::Length(12), Constraint::Length(20), Constraint::Length(6), Constraint::Length(10), Constraint::Length(8)])
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(" ⬡ Block Devices ", Style::default().fg(Theme::ACCENT).bold()))
            .style(Style::default().bg(Theme::BG)))
        .row_highlight_style(Style::default().fg(Theme::BG).bg(Theme::ACCENT));
    frame.render_widget(table, chunks[0]);

    // Right panel: drive details
    let detail_text = if let Some(d) = app.drives.get(app.drive_cursor) {
        let system_tag = if d.is_system { " ⚠ SYSTEM DRIVE — PROTECTED " } else { " ✓ Safe to sanitize " };
        let system_col = if d.is_system { Theme::DANGER } else { Theme::SUCCESS };
        vec![
            Line::raw(""),
            Line::from(Span::styled(format!("  /dev/{}", d.name), Style::default().fg(Theme::ACCENT).bold())),
            Line::raw(""),
            Line::from(vec![Span::styled("  Model:    ", Style::default().fg(Theme::MUTED)), Span::styled(&d.model, Style::default().fg(Theme::TEXT))]),
            Line::from(vec![Span::styled("  Type:     ", Style::default().fg(Theme::MUTED)), Span::styled(&d.drive_type, Style::default().fg(Theme::TEXT))]),
            Line::from(vec![Span::styled("  Size:     ", Style::default().fg(Theme::MUTED)), Span::styled(format_bytes(d.size_bytes), Style::default().fg(Theme::TEXT))]),
            Line::from(vec![Span::styled("  Path:     ", Style::default().fg(Theme::MUTED)), Span::styled(&d.path, Style::default().fg(Theme::TEXT))]),
            Line::raw(""),
            Line::from(Span::styled(system_tag, Style::default().fg(system_col).bold())),
            Line::raw(""),
            Line::from(vec![Span::styled("  Enter ", Style::default().fg(Theme::ACCENT).bold()), Span::styled("→ Select for Wipe", Style::default().fg(Theme::MUTED))]),
            Line::from(vec![Span::styled("  R     ", Style::default().fg(Theme::ACCENT).bold()), Span::styled("→ Refresh list", Style::default().fg(Theme::MUTED))]),
        ]
    } else {
        vec![Line::from(Span::styled("  No drives detected. Run with sudo.", Style::default().fg(Theme::WARNING)))]
    };

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Drive Info ", Style::default().fg(Theme::ACCENT2).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    let detail_para = Paragraph::new(detail_text).block(detail_block);
    frame.render_widget(detail_para, chunks[1]);
}

fn format_bytes(n: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = n as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 { size /= 1024.0; unit_idx += 1; }
    format!("{:.1} {}", size, units[unit_idx])
}
