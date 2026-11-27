use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Style, Stylize},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap}, text::{Line, Span}};
use crate::{app::{App, WipeMethod, WipePhase}, theme::Theme};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main[0]);

    // Method selector
    let methods = WipeMethod::all();
    let method_items: Vec<ListItem> = methods.iter().enumerate().map(|(i, m)| {
        let selected = app.wipe_method_cursor == i;
        let radio = if selected { "◉" } else { "○" };
        let style = if selected {
            Style::default().fg(Theme::ACCENT).bold()
        } else {
            Style::default().fg(Theme::TEXT)
        };
        ListItem::new(Line::from(Span::styled(format!("  {} {}", radio, m.label()), style)))
    }).collect();

    let method_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Sanitization Method ", Style::default().fg(Theme::ACCENT).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    let method_list = List::new(method_items).block(method_block);
    frame.render_widget(method_list, top[0]);

    // Right: config + target
    let verify_str = if app.wipe_verify { "[✓] Enabled" } else { "[ ] Disabled" };
    let verify_col = if app.wipe_verify { Theme::SUCCESS } else { Theme::MUTED };
    let expert_str = if app.wipe_expert { "[✓] Expert Mode" } else { "[ ] Standard Mode" };
    let expert_col = if app.wipe_expert { Theme::WARNING } else { Theme::MUTED };

    let selected_drive_text = if let Some(idx) = app.selected_drive {
        if let Some(d) = app.drives.get(idx) {
            format!("/dev/{} — {} ({})", d.name, d.model, format_bytes(d.size_bytes))
        } else { "No drive selected".to_string() }
    } else {
        "⚠  No drive selected — go to F2 Drives and select one".to_string()
    };

    let target_color = if app.selected_drive.is_some() { Theme::SUCCESS } else { Theme::WARNING };

    let config_lines = vec![
        Line::raw(""),
        Line::from(vec![Span::styled("  Target Drive: ", Style::default().fg(Theme::MUTED)),
            Span::styled(&selected_drive_text, Style::default().fg(target_color).bold())]),
        Line::raw(""),
        Line::from(vec![Span::styled("  Method: ", Style::default().fg(Theme::MUTED)),
            Span::styled(app.selected_method().label(), Style::default().fg(Theme::ACCENT).bold())]),
        Line::raw(""),
        Line::from(vec![Span::styled("  Post-Wipe Verify: ", Style::default().fg(Theme::MUTED)),
            Span::styled(verify_str, Style::default().fg(verify_col))]),
        Line::from(vec![Span::styled("  Mode: ", Style::default().fg(Theme::MUTED)),
            Span::styled(expert_str, Style::default().fg(expert_col))]),
        Line::raw(""),
        Line::from(Span::styled("  ─────────────────────────────", Style::default().fg(Theme::BORDER))),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  V ", Style::default().fg(Theme::ACCENT).bold()),
            Span::styled("Toggle Verify   ", Style::default().fg(Theme::MUTED)),
            Span::styled("E ", Style::default().fg(Theme::WARNING).bold()),
            Span::styled("Toggle Expert", Style::default().fg(Theme::MUTED)),
        ]),
        Line::from(vec![
            Span::styled("  Enter ", Style::default().fg(Theme::DANGER).bold()),
            Span::styled("→ Execute Wipe", Style::default().fg(Theme::MUTED)),
        ]),
    ];

    let config_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Wipe Configuration ", Style::default().fg(Theme::ACCENT2).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    let config_para = Paragraph::new(config_lines).block(config_block).wrap(Wrap { trim: false });
    frame.render_widget(config_para, top[1]);

    // Progress area
    render_progress(frame, main[1], app);
}

fn render_progress(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Progress ", Style::default().fg(Theme::ACCENT).bold()))
        .style(Style::default().bg(Theme::BG));

    match &app.wipe_phase {
        WipePhase::Idle => {
            let para = Paragraph::new(" Ready — select a drive and press Enter to begin")
                .block(block).style(Style::default().fg(Theme::MUTED));
            frame.render_widget(para, area);
        }
        WipePhase::Running { pass, total_passes, bytes_done, bytes_total, speed_mbps, started } => {
            let inner = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
                .margin(1)
                .split(block.inner(area));
            frame.render_widget(block, area);

            let percent = if *bytes_total > 0 { (*bytes_done * 100 / bytes_total) as u16 } else { 0 };
            let elapsed = started.elapsed().as_secs();
            let eta = if *speed_mbps > 0.0 {
                let remaining_bytes = bytes_total.saturating_sub(*bytes_done);
                (remaining_bytes as f64 / (*speed_mbps * 1_048_576.0)) as u64
            } else { 0 };

            let phase_info = Paragraph::new(format!(
                " Pass {}/{} │ {:.1} MB/s │ ETA: {}s │ Elapsed: {}s",
                pass, total_passes, speed_mbps, eta, elapsed
            )).style(Style::default().fg(Theme::TEXT));
            frame.render_widget(phase_info, inner[0]);

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(Theme::ACCENT).bg(Theme::BORDER))
                .percent(percent)
                .label(format!("{:.1} / {:.1} GB", *bytes_done as f64 / 1e9, *bytes_total as f64 / 1e9));
            frame.render_widget(gauge, inner[1]);
        }
        WipePhase::Verifying => {
            let para = Paragraph::new(" ⟳ Verifying wipe integrity...")
                .block(block).style(Style::default().fg(Theme::WARNING).bold());
            frame.render_widget(para, area);
        }
        WipePhase::Done { success, elapsed } => {
            let (msg, color) = if *success {
                (format!(" ✓ Wipe COMPLETE — {} elapsed. All data securely erased.", format_dur(elapsed.as_secs())), Theme::SUCCESS)
            } else {
                (" ✗ Wipe FAILED — see log for details".to_string(), Theme::DANGER)
            };
            let block2 = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .title(Span::styled(" Result ", Style::default().fg(color).bold()))
                .style(Style::default().bg(Theme::BG));
            let para = Paragraph::new(msg).block(block2).style(Style::default().fg(color).bold());
            frame.render_widget(para, area);
        }
        WipePhase::Confirming => {
            let para = Paragraph::new(" Confirm wipe in the dialog above")
                .block(block).style(Style::default().fg(Theme::WARNING));
            frame.render_widget(para, area);
        }
        WipePhase::Error(e) => {
            let para = Paragraph::new(format!(" Error: {}", e))
                .block(block).style(Style::default().fg(Theme::DANGER).bold());
            frame.render_widget(para, area);
        }
    }
}

fn format_bytes(n: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = n as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 { size /= 1024.0; unit_idx += 1; }
    format!("{:.1} {}", size, units[unit_idx])
}

fn format_dur(secs: u64) -> String {
    let m = secs / 60; let s = secs % 60;
    if m > 0 { format!("{}m {}s", m, s) } else { format!("{}s", s) }
}
