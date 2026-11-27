use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, Wrap}, text::{Line, Span}};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, _app: &crate::app::App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let keybinds = vec![
        Line::raw(""),
        Line::from(Span::styled(" Global", Style::default().fg(Theme::ACCENT).bold())),
        Line::from(Span::styled(" ─────────────────────────────", Style::default().fg(Theme::BORDER))),
        kv("F1", "Dashboard"),
        kv("F2", "Drive Manager"),
        kv("F3", "Wipe Wizard"),
        kv("F4", "Forensic Carver"),
        kv("F5", "This Help screen"),
        kv("Q / Ctrl+C", "Quit SecureForge"),
        Line::raw(""),
        Line::from(Span::styled(" Drive Manager", Style::default().fg(Theme::ACCENT).bold())),
        Line::from(Span::styled(" ─────────────────────────────", Style::default().fg(Theme::BORDER))),
        kv("↑ / ↓", "Navigate drives"),
        kv("Enter", "Select drive for wipe"),
        kv("R", "Refresh drive list"),
        Line::raw(""),
        Line::from(Span::styled(" Wipe Wizard", Style::default().fg(Theme::ACCENT).bold())),
        Line::from(Span::styled(" ─────────────────────────────", Style::default().fg(Theme::BORDER))),
        kv("↑ / ↓", "Select wipe method"),
        kv("V", "Toggle post-wipe verify"),
        kv("E", "Toggle expert mode"),
        kv("Enter", "Confirm and start wipe"),
        kv("Y / N", "Confirm / Cancel popup"),
    ];

    let kb_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⌨  Keyboard Shortcuts ", Style::default().fg(Theme::ACCENT).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    let kb_para = Paragraph::new(keybinds).block(kb_block).wrap(Wrap { trim: false });
    frame.render_widget(kb_para, chunks[0]);

    let about_lines = vec![
        Line::raw(""),
        Line::from(Span::styled(" SecureForge v0.1.0", Style::default().fg(Theme::ACCENT).bold())),
        Line::raw(""),
        Line::from(Span::styled(" Integrated Secure Data Erasure &", Style::default().fg(Theme::TEXT))),
        Line::from(Span::styled(" Forensic File Recovery Platform", Style::default().fg(Theme::TEXT))),
        Line::raw(""),
        Line::from(Span::styled(" Standards Compliance:", Style::default().fg(Theme::MUTED))),
        Line::from(Span::styled("   • NIST SP 800-88 Rev 1", Style::default().fg(Theme::SUCCESS))),
        Line::from(Span::styled("   • DoD 5220.22-M (3 & 7 pass)", Style::default().fg(Theme::SUCCESS))),
        Line::from(Span::styled("   • Gutmann 35-pass", Style::default().fg(Theme::SUCCESS))),
        Line::raw(""),
        Line::from(Span::styled(" License: MIT", Style::default().fg(Theme::MUTED))),
        Line::from(Span::styled(" Built for SIH-149", Style::default().fg(Theme::MUTED))),
        Line::raw(""),
        Line::from(Span::styled(" ⚠  Always run with sudo for block", Style::default().fg(Theme::WARNING))),
        Line::from(Span::styled("    device access.", Style::default().fg(Theme::WARNING))),
    ];

    let about_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⬡ About SecureForge ", Style::default().fg(Theme::ACCENT2).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    let about_para = Paragraph::new(about_lines).block(about_block).wrap(Wrap { trim: false });
    frame.render_widget(about_para, chunks[1]);
}

fn kv(key: &str, val: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:15}", key), Style::default().fg(Theme::ACCENT).bold()),
        Span::styled(val.to_string(), Style::default().fg(Theme::TEXT)),
    ])
}
