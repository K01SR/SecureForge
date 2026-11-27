use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    text::{Line, Span},
};
use crate::{app::{App, LogLevel}, theme::Theme};
use super::format_bytes;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(0)])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Length(11), Constraint::Min(0)])
        .split(cols[0]);

    render_logo(frame, left[0]);
    render_stats(frame, left[1], app);
    render_quicknav(frame, left[2]);
    render_log(frame, cols[1], app);
}

fn render_logo(frame: &mut Frame, area: Rect) {
    let logo = vec![
        Line::from(Span::styled("  ┌──────────────────────────────┐", Style::default().fg(Theme::BORDER))),
        Line::from(vec![
            Span::styled("  │ ", Style::default().fg(Theme::BORDER)),
            Span::styled("▓▓▓ SECUREFORGE ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("v0.1.0 │", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  │ ", Style::default().fg(Theme::BORDER)),
            Span::styled("Sanitize · Recover · Certify ", Style::default().fg(Theme::MUTED)),
            Span::styled("│", Style::default().fg(Theme::BORDER)),
        ]),
        Line::from(vec![
            Span::styled("  │ ", Style::default().fg(Theme::BORDER)),
            Span::styled("NIST SP 800-88 R1 Compliant   ", Style::default().fg(Theme::INDIGO)),
            Span::styled("│", Style::default().fg(Theme::BORDER)),
        ]),
        Line::from(Span::styled("  └──────────────────────────────┘", Style::default().fg(Theme::BORDER))),
    ];
    let block = Block::default().style(Style::default().bg(Theme::BG));
    let para = Paragraph::new(logo).block(block);
    frame.render_widget(para, area);
}

fn render_stats(frame: &mut Frame, area: Rect, app: &App) {
    let total = app.drives.len();
    let sys   = app.drives.iter().filter(|d| d.is_system).count();
    let ssd   = app.drives.iter().filter(|d| d.drive_type == "SSD" || d.drive_type == "NVMe").count();
    let cap: u64 = app.drives.iter().map(|d| d.size_bytes).sum();

    fn stat_line<'a>(label: &'a str, val: String, color: ratatui::style::Color) -> Line<'a> {
        Line::from(vec![
            Span::styled(format!("  {:<18}", label), Style::default().fg(Theme::TEXT_DIM)),
            Span::styled(val, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ])
    }

    let lines = vec![
        Line::raw(""),
        stat_line("Drives detected:", total.to_string(), Theme::CYAN),
        stat_line("System drives:", sys.to_string(), if sys > 0 { Theme::WARNING } else { Theme::SUCCESS }),
        stat_line("SSD / NVMe:", ssd.to_string(), Theme::BLUE),
        stat_line("Total capacity:", format_bytes(cap), Theme::SUCCESS),
        Line::raw(""),
        stat_line("Standard:", "NIST SP 800-88 R1".to_string(), Theme::PURPLE),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⚡ System Overview ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn render_quicknav(frame: &mut Frame, area: Rect) {
    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled(" F2 ", Style::default().fg(Theme::BG).bg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("  Drive Browser & Selection", Style::default().fg(Theme::TEXT)),
        ])),
        ListItem::new(Line::raw("")),
        ListItem::new(Line::from(vec![
            Span::styled(" F3 ", Style::default().fg(Theme::BG).bg(Theme::PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled("  Entropy Heatmap Analyzer", Style::default().fg(Theme::TEXT)),
        ])),
        ListItem::new(Line::raw("")),
        ListItem::new(Line::from(vec![
            Span::styled(" F4 ", Style::default().fg(Theme::BG).bg(Theme::DANGER).add_modifier(Modifier::BOLD)),
            Span::styled("  Data Sanitization Wizard", Style::default().fg(Theme::TEXT)),
        ])),
        ListItem::new(Line::raw("")),
        ListItem::new(Line::from(vec![
            Span::styled(" F5 ", Style::default().fg(Theme::BG).bg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::styled("  Forensic File Carver", Style::default().fg(Theme::TEXT)),
        ])),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⬡ Quick Navigation ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_log(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.log.iter().rev().take(50).map(|(level, msg)| {
        let (icon, color) = match level {
            LogLevel::Info    => ("·", Theme::TEXT_DIM),
            LogLevel::Success => ("✓", Theme::SUCCESS),
            LogLevel::Warning => ("⚠", Theme::WARNING),
            LogLevel::Error   => ("✗", Theme::DANGER),
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(color)),
            Span::styled(msg.as_str(), Style::default().fg(color)),
        ]))
    }).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ░ Audit Event Log ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::BG));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
