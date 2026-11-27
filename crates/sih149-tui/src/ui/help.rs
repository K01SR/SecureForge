use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    text::{Line, Span},
};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, _app: &crate::app::App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_keybinds(frame, cols[0]);
    render_about(frame, cols[1]);
}

fn render_keybinds(frame: &mut Frame, area: Rect) {
    let sections: &[(&str, &[(&str, &str)])] = &[
        ("Global", &[
            ("F1", "Dashboard — stats & log"),
            ("F2", "Drive Browser & selector"),
            ("F3", "Entropy Heatmap Analyzer"),
            ("F4", "Data Sanitization Wizard"),
            ("F5", "Forensic File Carver"),
            ("F6", "Help & About (this page)"),
            ("Q / Ctrl+C", "Quit SecureForge"),
        ]),
        ("Drive Browser (F2)", &[
            ("↑ / ↓", "Navigate drives"),
            ("Enter", "Select drive for wipe"),
            ("E", "Open entropy view for drive"),
            ("R", "Refresh drive list"),
        ]),
        ("Entropy View (F3)", &[
            ("↑ / ↓", "Switch drive"),
            ("E", "Load / reload entropy data"),
        ]),
        ("Wipe Wizard (F4)", &[
            ("↑ / ↓", "Select sanitization method"),
            ("V", "Toggle post-wipe verify"),
            ("E", "Toggle expert mode"),
            ("Enter", "Confirm & start wipe"),
            ("Y / N", "Confirm / cancel dialog"),
        ]),
        ("Forensic Carver (F5)", &[
            ("Tab", "Cycle between fields"),
            ("↑ / ↓", "Adjust min confidence"),
            ("Type", "Edit current field"),
            ("Backspace", "Delete character"),
            ("Enter", "Start carving scan"),
            ("↑ / ↓", "Navigate results table"),
        ]),
    ];

    let mut lines = vec![Line::raw("")];
    for (section, keys) in sections {
        lines.push(Line::from(Span::styled(
            format!("  {}", section),
            Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  ──────────────────────────────────────",
            Style::default().fg(Theme::BORDER),
        )));
        for (key, desc) in *keys {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:15}", key), Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)),
                Span::styled(*desc, Style::default().fg(Theme::TEXT)),
            ]));
        }
        lines.push(Line::raw(""));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⌨  Keyboard Reference ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_about(frame: &mut Frame, area: Rect) {
    let entropy_legend: &[(&str, ratatui::style::Color, &str)] = &[
        ("░  0–1 bits/byte", Theme::ENT_0, "Dead / Zeroed sectors"),
        ("░  1–2 bits/byte", Theme::ENT_1, "Near-zero (mostly empty)"),
        ("▒  2–4 bits/byte", Theme::ENT_2, "Low entropy (text, FS structures)"),
        ("▒  4–6 bits/byte", Theme::ENT_3, "Moderate (typical data files)"),
        ("▓  6–7 bits/byte", Theme::ENT_4, "High (compressed data)"),
        ("█  7–8 bits/byte", Theme::ENT_5, "Encrypted / compressed"),
    ];

    let mut lines = vec![
        Line::raw(""),
        Line::from(Span::styled("  ▓ SecureForge v0.1.0", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))),
        Line::raw(""),
        Line::from(Span::styled("  Integrated Secure Data Erasure &", Style::default().fg(Theme::TEXT))),
        Line::from(Span::styled("  Forensic File Recovery Platform", Style::default().fg(Theme::TEXT))),
        Line::raw(""),
        Line::from(Span::styled("  Standards Compliance:", Style::default().fg(Theme::MUTED))),
        Line::from(Span::styled("    ✓ NIST SP 800-88 Rev 1", Style::default().fg(Theme::SUCCESS))),
        Line::from(Span::styled("    ✓ DoD 5220.22-M  (3 & 7 pass)", Style::default().fg(Theme::SUCCESS))),
        Line::from(Span::styled("    ✓ Gutmann 35-pass", Style::default().fg(Theme::SUCCESS))),
        Line::raw(""),
        Line::from(Span::styled(
            "  ──────────────── Entropy Legend ─────────────────",
            Style::default().fg(Theme::BORDER),
        )),
        Line::raw(""),
    ];

    for (sym, color, desc) in entropy_legend {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<18}", sym), Style::default().fg(*color).add_modifier(Modifier::BOLD)),
            Span::styled(*desc, Style::default().fg(Theme::TEXT_DIM)),
        ]));
    }

    lines.extend([
        Line::raw(""),
        Line::from(Span::styled("  ──────────────────────────────────────────────────", Style::default().fg(Theme::BORDER))),
        Line::raw(""),
        Line::from(Span::styled("  ⚠ Always run with sudo for block device access.", Style::default().fg(Theme::WARNING))),
        Line::from(Span::styled("    Wipe operations are IRREVERSIBLE.", Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD))),
        Line::raw(""),
        Line::from(Span::styled("  Built for Smart India Hackathon 2024 — SIH-149", Style::default().fg(Theme::MUTED))),
        Line::from(Span::styled("  License: MIT", Style::default().fg(Theme::MUTED))),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⬡ About & Entropy Guide ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}
