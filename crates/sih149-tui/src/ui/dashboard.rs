use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Wrap},
    text::{Line, Span},
};
use crate::{app::{App, LogLevel}, theme::Theme};
use super::format_bytes;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    // 3-way layout: Top Banner, Middle Grid (Stats + Quick Actions + Live Drives), Bottom Grid (Security Status + Audit Log)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Header / Hero telemetry banner
            Constraint::Length(10), // Metrics & Drives snapshot
            Constraint::Min(0),    // Security & Live Audit stream
        ])
        .split(area);

    render_hero_banner(frame, chunks[0], app);
    render_middle_metrics(frame, chunks[1], app);
    render_bottom_split(frame, chunks[2], app);
}

fn render_hero_banner(frame: &mut Frame, area: Rect, app: &App) {
    let hero_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(34),
            Constraint::Min(0),
            Constraint::Length(38),
        ])
        .split(area);

    // 1. ASCII Hologram
    let logo = vec![
        Line::from(Span::styled("  ▄▄▄▄▄ ▄▄▄▄▄ ▄▄▄▄▄ ▄   ▄ ▄▄▄▄ ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  █▄▄▄▄ █▄▄▄█ █   █ █   █ █▄▄▄▄", Style::default().fg(Theme::BLUE))),
        Line::from(Span::styled("  ▄▄▄▄█ █     █▄▄▄█ █▄▄▄█ █▄▄▄▄", Style::default().fg(Theme::PURPLE))),
        Line::from(vec![
            Span::styled("   FORENSIC SANITIZATION ENGINE ", Style::default().fg(Theme::TEXT_DIM)),
            Span::styled("v0.1.0", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        ]),
    ];
    let logo_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .style(Style::default().bg(Theme::SURFACE));
    frame.render_widget(Paragraph::new(logo).block(logo_block), hero_layout[0]);

    // 2. Live Engine Status & Security Telemetry
    let pulse_char = ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"][(app.tick / 2) as usize % 10];
    let total_drives = app.drives.len();
    let sys_drives = app.drives.iter().filter(|d| d.is_system).count();
    let safe_drives = total_drives.saturating_sub(sys_drives);

    let center_lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!("  {} SYSTEM CORE STATUS: ", pulse_char), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("ONLINE (DEFENSE-GRADE)", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::styled(" │ KERNEL: ", Style::default().fg(Theme::BORDER)),
            Span::styled("LINUX POSIX / NOFOLLOW ACTIVE", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("  ▸ Standards Verified: ", Style::default().fg(Theme::MUTED)),
            Span::styled("NIST SP 800-88 Rev 1 (Clear/Purge)", Style::default().fg(Theme::SUCCESS)),
            Span::styled("  DoD 5220.22-M (3/7-Pass)", Style::default().fg(Theme::INFO)),
            Span::styled("  Gutmann 35-Pass", Style::default().fg(Theme::PURPLE)),
        ]),
        Line::from(vec![
            Span::styled("  ▸ Storage Topology:   ", Style::default().fg(Theme::MUTED)),
            Span::styled(format!("{} Block Devices ({} Sanitizable Target{}, {} Protected System)", 
                total_drives, safe_drives, if safe_drives == 1 {""} else {"s"}, sys_drives), 
                Style::default().fg(Theme::TEXT)),
        ]),
    ];
    let center_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ⚡ Operational Telemetry ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    frame.render_widget(Paragraph::new(center_lines).block(center_block), hero_layout[1]);

    // 3. Cryptographic Signature & Verification Module
    let crypto_lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  CSPRNG Source:  ", Style::default().fg(Theme::MUTED)),
            Span::styled("OS /dev/urandom (OsRng)", Style::default().fg(Theme::SUCCESS)),
        ]),
        Line::from(vec![
            Span::styled("  Hash Security:  ", Style::default().fg(Theme::MUTED)),
            Span::styled("BLAKE3 / SHA-256 Chain", Style::default().fg(Theme::CYAN)),
        ]),
        Line::from(vec![
            Span::styled("  Anti-Forensics: ", Style::default().fg(Theme::MUTED)),
            Span::styled("Multi-Journal Rename Storm", Style::default().fg(Theme::PURPLE)),
        ]),
    ];
    let crypto_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" 🔒 Cryptographic Rigor ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    frame.render_widget(Paragraph::new(crypto_lines).block(crypto_block), hero_layout[2]);
}

fn render_middle_metrics(frame: &mut Frame, area: Rect, app: &App) {
    let middle_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(34),
            Constraint::Length(38),
            Constraint::Min(0),
        ])
        .split(area);

    // Card 1: Key Metrics Gauges
    let total_bytes: u64 = app.drives.iter().map(|d| d.size_bytes).sum();
    let sys_count = app.drives.iter().filter(|d| d.is_system).count();
    let total_count = app.drives.len().max(1);
    let safe_ratio = ((total_count - sys_count) * 100 / total_count) as u16;

    let sub_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .margin(1)
        .split(middle_layout[0]);

    let metrics_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" 📊 Storage Capacity ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    frame.render_widget(metrics_block, middle_layout[0]);

    let cap_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Theme::CYAN).bg(Theme::BORDER))
        .percent(100)
        .label(format!("Total: {}", format_bytes(total_bytes)));
    frame.render_widget(cap_gauge, sub_area[0]);

    let target_gauge = Gauge::default()
        .gauge_style(Style::default().fg(Theme::SUCCESS).bg(Theme::BORDER))
        .percent(safe_ratio)
        .label(format!("Sanitizable: {}%", safe_ratio));
    frame.render_widget(target_gauge, sub_area[1]);

    // Card 2: Interactive Function Shortcuts
    let shortcuts = vec![
        ListItem::new(Line::from(vec![
            Span::styled(" [F2] ", Style::default().fg(Theme::BG).bg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(" Drive Manager     ", Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
            Span::styled("Inspect & mount points", Style::default().fg(Theme::MUTED)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(" [F3] ", Style::default().fg(Theme::BG).bg(Theme::PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled(" Entropy Heatmap   ", Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
            Span::styled("Multi-color sector view", Style::default().fg(Theme::MUTED)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(" [F4] ", Style::default().fg(Theme::BG).bg(Theme::DANGER).add_modifier(Modifier::BOLD)),
            Span::styled(" Sanitizer Wizard  ", Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
            Span::styled("DoD/NIST/Gutmann wipe", Style::default().fg(Theme::MUTED)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled(" [F5] ", Style::default().fg(Theme::BG).bg(Theme::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::styled(" Forensic Carver   ", Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD)),
            Span::styled("Recover deleted data", Style::default().fg(Theme::MUTED)),
        ])),
    ];
    let nav_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" 🚀 Mission Control Shortcuts ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let nav_list = List::new(shortcuts).block(nav_block);
    frame.render_widget(nav_list, middle_layout[1]);

    // Card 3: Live Drive Table Mini-Snapshot
    let drive_rows: Vec<Row> = app.drives.iter().take(4).map(|d| {
        let (status_text, status_color) = if d.is_system {
            ("PROT (SYS)", Theme::DANGER)
        } else {
            ("TARGET READY", Theme::SUCCESS)
        };
        Row::new(vec![
            Cell::from(format!("/dev/{}", d.name)).style(Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Cell::from(d.model.chars().take(15).collect::<String>()).style(Style::default().fg(Theme::TEXT_DIM)),
            Cell::from(d.drive_type.clone()).style(Style::default().fg(Theme::BLUE)),
            Cell::from(format_bytes(d.size_bytes)).style(Style::default().fg(Theme::TEXT)),
            Cell::from(status_text).style(Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]).height(1)
    }).collect();

    let drive_table = Table::new(drive_rows, [
        Constraint::Length(11),
        Constraint::Length(16),
        Constraint::Length(7),
        Constraint::Length(10),
        Constraint::Min(0),
    ])
    .header(Row::new(vec![
        Cell::from("Path").style(Style::default().fg(Theme::MUTED)),
        Cell::from("Model").style(Style::default().fg(Theme::MUTED)),
        Cell::from("Type").style(Style::default().fg(Theme::MUTED)),
        Cell::from("Capacity").style(Style::default().fg(Theme::MUTED)),
        Cell::from("Sanitize State").style(Style::default().fg(Theme::MUTED)),
    ]))
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" 💽 Detected Devices Snapshot ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE)));
    frame.render_widget(drive_table, middle_layout[2]);
}

fn render_bottom_split(frame: &mut Frame, area: Rect, app: &App) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(45),
            Constraint::Min(0),
        ])
        .split(area);

    // Left: Security Posture & Engine Guarantees
    let security_guarantees = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  🛡  Defense-in-Depth File Shredding:", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("     • Strict canonicalization & prefix containment (/etc, /boot, /dev)", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("     • lstat() symlink detection — symlinks are never followed", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("     • O_NOFOLLOW open flag preventing TOCTOU attack vector", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("     • sync_all() flush ensuring non-volatile NAND/platter write", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("     • Multi-pass journal scrub with 8 random hex renames", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  🎯 Post-Sanitization Entropy Verification:", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("     • Full Shannon entropy check on PRNG passes (threshold ≥ 7.9 b/B)", Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::from(vec![
            Span::styled("     • Bitwise verification on zero/one deterministic passes", Style::default().fg(Theme::TEXT_DIM)),
        ]),
    ];
    let sec_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" 🛡 Sanitization & Security Posture ", Style::default().fg(Theme::SUCCESS).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    frame.render_widget(Paragraph::new(security_guarantees).block(sec_block).wrap(Wrap { trim: true }), split[0]);

    // Right: Live Event Audit Stream
    let items: Vec<ListItem> = app.log.iter().rev().take(60).map(|(level, msg)| {
        let (icon, color, tag) = match level {
            LogLevel::Info    => ("·", Theme::TEXT_DIM, "INFO"),
            LogLevel::Success => ("✓", Theme::SUCCESS, "PASS"),
            LogLevel::Warning => ("⚠", Theme::WARNING, "WARN"),
            LogLevel::Error   => ("✗", Theme::DANGER, "FAIL"),
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!(" [{:<4}] ", tag), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", icon), Style::default().fg(color)),
            Span::styled(msg.as_str(), Style::default().fg(Theme::TEXT)),
        ]))
    }).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" 📜 Live Forensic Audit Log ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::BG));
    let list = List::new(items).block(block);
    frame.render_widget(list, split[1]);
}
