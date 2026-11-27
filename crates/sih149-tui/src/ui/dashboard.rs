use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Style, Stylize},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}, text::{Line, Span}};
use crate::{app::App, theme::Theme};

const LOGO: &str = r#"
 ▗▄▄▖▗▄▄▄▖▗▄▄▖  
▐▌   ▐▌  ▐▌   ▐ 
 ▝▀▚▖▐▛▀▀▘▐▌   ▐ 
▗▄▄▞▘▐▙▄▄▖▝▚▄▄▖▐ 
"#;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Min(0)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Length(10), Constraint::Min(0)])
        .split(cols[0]);

    // Logo
    let logo_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::ACCENT))
        .style(Style::default().bg(Theme::SURFACE));
    let logo_para = Paragraph::new(LOGO)
        .block(logo_block)
        .style(Style::default().fg(Theme::ACCENT).bold());
    frame.render_widget(logo_para, left[0]);

    // Stats
    let total_drives = app.drives.len();
    let system_drives = app.drives.iter().filter(|d| d.is_system).count();
    let total_bytes: u64 = app.drives.iter().map(|d| d.size_bytes).sum();
    let capacity_str = format_bytes(total_bytes);

    let stats_items = vec![
        Line::from(vec![
            Span::styled("  Drives Detected: ", Style::default().fg(Theme::MUTED)),
            Span::styled(total_drives.to_string(), Style::default().fg(Theme::ACCENT).bold()),
        ]),
        Line::from(vec![
            Span::styled("  System Drives:   ", Style::default().fg(Theme::MUTED)),
            Span::styled(system_drives.to_string(), Style::default().fg(Theme::WARNING).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Total Capacity:  ", Style::default().fg(Theme::MUTED)),
            Span::styled(capacity_str, Style::default().fg(Theme::SUCCESS).bold()),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Standard:        ", Style::default().fg(Theme::MUTED)),
            Span::styled("NIST SP 800-88 R1", Style::default().fg(Theme::ACCENT2).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Version:         ", Style::default().fg(Theme::MUTED)),
            Span::styled("SecureForge v0.1.0", Style::default().fg(Theme::TEXT)),
        ]),
    ];

    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⚡ System Status ", Style::default().fg(Theme::ACCENT).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    let stats_para = Paragraph::new(stats_items).block(stats_block);
    frame.render_widget(stats_para, left[1]);

    // Quick actions
    let actions = vec![
        Line::from(vec![Span::styled(" F2 ", Style::default().fg(Theme::BG).bg(Theme::ACCENT).bold()), Span::styled(" Browse Drives", Style::default().fg(Theme::TEXT))]),
        Line::raw(""),
        Line::from(vec![Span::styled(" F3 ", Style::default().fg(Theme::BG).bg(Theme::DANGER).bold()), Span::styled(" Sanitize Drive", Style::default().fg(Theme::TEXT))]),
        Line::raw(""),
        Line::from(vec![Span::styled(" F4 ", Style::default().fg(Theme::BG).bg(Theme::SUCCESS).bold()), Span::styled(" Forensic Carver", Style::default().fg(Theme::TEXT))]),
        Line::raw(""),
        Line::from(vec![Span::styled(" F5 ", Style::default().fg(Theme::BG).bg(Theme::MUTED).bold()), Span::styled(" Help & About", Style::default().fg(Theme::TEXT))]),
    ];

    let actions_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⬡ Quick Actions ", Style::default().fg(Theme::ACCENT2).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    let actions_para = Paragraph::new(actions).block(actions_block).wrap(Wrap { trim: false });
    frame.render_widget(actions_para, left[2]);

    // Right: event log
    let log_items: Vec<ListItem> = app.log.iter().rev().take(40).map(|l| {
        let color = if l.contains("Error") || l.contains("FAILED") { Theme::DANGER }
            else if l.contains("OK") || l.contains("success") || l.contains("complete") { Theme::SUCCESS }
            else if l.contains("Warn") { Theme::WARNING }
            else { Theme::TEXT };
        ListItem::new(Line::from(Span::styled(format!(" › {}", l), Style::default().fg(color))))
    }).collect();

    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ░ Audit Event Log ", Style::default().fg(Theme::ACCENT).bold()))
        .style(Style::default().bg(Theme::BG));
    let log_list = List::new(log_items).block(log_block);
    frame.render_widget(log_list, cols[1]);
}

fn format_bytes(n: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = n as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 { size /= 1024.0; unit_idx += 1; }
    format!("{:.1} {}", size, units[unit_idx])
}
