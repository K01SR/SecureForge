use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    text::{Line, Span},
};
use crate::{app::{App, WipeMethod, WipePhase}, theme::Theme};
use super::{format_bytes, format_dur};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(7)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(main[0]);

    render_method_list(frame, top[0], app);
    render_config(frame, top[1], app);
    render_progress(frame, main[1], app);
}

fn render_method_list(frame: &mut Frame, area: Rect, app: &App) {
    let methods = WipeMethod::all();
    let items: Vec<ListItem> = methods.iter().enumerate().map(|(i, m)| {
        let sel = app.wipe_method_cursor == i;
        let (risk_lbl, risk_lvl) = m.risk_label();
        let risk_color = match risk_lvl {
            1 => Theme::SUCCESS,
            2 => Theme::INFO,
            3 => Theme::WARNING,
            4 => Theme::DANGER,
            _ => Theme::DANGER,
        };
        let radio = if sel { "◉" } else { "○" };
        let radio_color = if sel { Theme::CYAN } else { Theme::MUTED };
        let bg = if sel { Theme::SURFACE2 } else { Theme::BG };

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!("  {} ", radio), Style::default().fg(radio_color)),
                Span::styled(
                    m.label(),
                    Style::default()
                        .fg(if sel { Theme::CYAN } else { Theme::TEXT })
                        .add_modifier(if sel { Modifier::BOLD } else { Modifier::empty() }),
                ),
            ]),
            Line::from(vec![
                Span::styled("       ", Style::default()),
                Span::styled(format!("{} pass{}", m.passes(), if m.passes() == 1 { "" } else { "es" }), Style::default().fg(Theme::MUTED)),
                Span::styled("  │  ", Style::default().fg(Theme::BORDER)),
                Span::styled(risk_lbl, Style::default().fg(risk_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::raw(""),
        ]).style(Style::default().bg(bg))
    }).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Sanitization Method  [↑↓] Select ",
            Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::BG));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_config(frame: &mut Frame, area: Rect, app: &App) {
    let top = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Target + toggles
    let (target_line, target_color) = match app.selected_drive.and_then(|i| app.drives.get(i)) {
        Some(d) => (
            format!("/dev/{}  —  {}  ({})", d.name, d.model.chars().take(20).collect::<String>(), format_bytes(d.size_bytes)),
            if d.is_system { Theme::DANGER } else { Theme::SUCCESS },
        ),
        None => (
            "No drive selected — go to F2 and press Enter".to_string(),
            Theme::WARNING,
        ),
    };

    let verify_str = if app.wipe_verify { " ✓ Enabled " } else { " ✗ Disabled " };
    let verify_bg  = if app.wipe_verify { Theme::SUCCESS } else { Theme::MUTED };
    let expert_str = if app.wipe_expert { " ⚡ Expert " } else { " Standard " };
    let expert_bg  = if app.wipe_expert { Theme::WARNING } else { Theme::BORDER };

    let config_lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Target: ", Style::default().fg(Theme::MUTED)),
            Span::styled(&target_line, Style::default().fg(target_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Method: ", Style::default().fg(Theme::MUTED)),
            Span::styled(app.selected_method().label(), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  ×{} passes", app.selected_method().passes()), Style::default().fg(Theme::MUTED)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Verify: ", Style::default().fg(Theme::MUTED)),
            Span::styled(verify_str, Style::default().fg(Theme::BG).bg(verify_bg).add_modifier(Modifier::BOLD)),
            Span::styled("  [V] toggle  ", Style::default().fg(Theme::MUTED)),
            Span::styled(expert_str, Style::default().fg(Theme::BG).bg(expert_bg).add_modifier(Modifier::BOLD)),
            Span::styled("  [E] toggle", Style::default().fg(Theme::MUTED)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  ──────────────────────────────────────────────────", Style::default().fg(Theme::BORDER)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  [Enter] ", Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD)),
            Span::styled("Execute Wipe    ", Style::default().fg(Theme::MUTED)),
            Span::styled("[F2] ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("Change Drive    ", Style::default().fg(Theme::MUTED)),
            Span::styled("[F3] ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)),
            Span::styled("Entropy View", Style::default().fg(Theme::MUTED)),
        ]),
    ];

    let config_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Wipe Configuration ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let config_para = Paragraph::new(config_lines).block(config_block).wrap(Wrap { trim: false });
    frame.render_widget(config_para, top[0]);

    // Risk indicator
    let (risk_lbl, risk_lvl) = app.selected_method().risk_label();
    let risk_pct = (risk_lvl as u16 * 20).min(100);
    let risk_color = match risk_lvl {
        1 => Theme::SUCCESS,
        2 => Theme::INFO,
        3 => Theme::WARNING,
        4 => Theme::DANGER,
        _ => Theme::DANGER,
    };
    let risk_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER))
            .title(Span::styled(" Risk Level ", Style::default().fg(Theme::TEXT_DIM)))
            .style(Style::default().bg(Theme::SURFACE)))
        .gauge_style(Style::default().fg(risk_color))
        .percent(risk_pct)
        .label(format!("  {}  — {} passes", risk_lbl, app.selected_method().passes()));
    frame.render_widget(risk_gauge, top[1]);
}

fn render_progress(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Progress ", Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::BG));

    match &app.wipe_phase {
        WipePhase::Idle => {
            let para = Paragraph::new("  Ready — configure above and press Enter to start")
                .block(block)
                .style(Style::default().fg(Theme::MUTED));
            frame.render_widget(para, area);
        }

        WipePhase::Running { pass, total_passes, bytes_done, bytes_total, speed_mbps, started } => {
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
                .margin(1)
                .split(inner);

            let pct = if *bytes_total > 0 { (*bytes_done * 100 / bytes_total).min(100) as u16 } else { 0 };
            let elapsed = started.elapsed().as_secs();
            let eta = if *speed_mbps > 0.5 {
                let rem = bytes_total.saturating_sub(*bytes_done);
                (rem as f64 / (*speed_mbps * 1_048_576.0)) as u64
            } else { 0 };

            // Info line
            let info = Line::from(vec![
                Span::styled(format!("  Pass {}/{}", pass, total_passes), Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
                Span::styled("  │  ", Style::default().fg(Theme::BORDER)),
                Span::styled(format!("{:.1} MB/s", speed_mbps), Style::default().fg(Theme::SUCCESS)),
                Span::styled("  │  ", Style::default().fg(Theme::BORDER)),
                Span::styled(format!("ETA {}", format_dur(eta)), Style::default().fg(Theme::WARNING)),
                Span::styled("  │  Elapsed ", Style::default().fg(Theme::MUTED)),
                Span::styled(format_dur(elapsed), Style::default().fg(Theme::TEXT)),
            ]);
            frame.render_widget(Paragraph::new(info), layout[0]);

            // Main gauge
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(Theme::CYAN).bg(Theme::BORDER))
                .percent(pct)
                .label(format!(
                    "  {:.2} / {:.2} GB  ({}%)",
                    *bytes_done as f64 / 1e9,
                    *bytes_total as f64 / 1e9,
                    pct,
                ));
            frame.render_widget(gauge, layout[1]);

            // Pass gauge
            let pass_pct = ((*pass as u16 - 1) * 100 / (*total_passes as u16).max(1)).min(100);
            let pass_gauge = Gauge::default()
                .gauge_style(Style::default().fg(Theme::PURPLE).bg(Theme::BORDER))
                .percent(pass_pct)
                .label(format!("  Passes: {}/{}", pass, total_passes));
            frame.render_widget(pass_gauge, layout[2]);
        }

        WipePhase::Verifying => {
            let para = Paragraph::new("  ⟳ Running post-wipe verification…")
                .block(block)
                .style(Style::default().fg(Theme::WARNING).add_modifier(Modifier::BOLD));
            frame.render_widget(para, area);
        }

        WipePhase::Done { success, elapsed } => {
            let (msg, color) = if *success {
                (format!("  ✓  Wipe COMPLETE — {} elapsed. All data securely erased per {}.", format_dur(elapsed.as_secs()), app.selected_method().label()), Theme::SUCCESS)
            } else {
                ("  ✗  Wipe FAILED — check audit log for details.".to_string(), Theme::DANGER)
            };
            let done_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .title(Span::styled(" Result ", Style::default().fg(color).add_modifier(Modifier::BOLD)))
                .style(Style::default().bg(Theme::BG));
            let para = Paragraph::new(msg)
                .block(done_block)
                .style(Style::default().fg(color).add_modifier(Modifier::BOLD));
            frame.render_widget(para, area);
        }

        WipePhase::Error(e) => {
            let para = Paragraph::new(format!("  ✗  Error: {}", e))
                .block(block)
                .style(Style::default().fg(Theme::DANGER).add_modifier(Modifier::BOLD));
            frame.render_widget(para, area);
        }

        WipePhase::Confirming => {
            let para = Paragraph::new("  Waiting for confirmation in the dialog above…")
                .block(block)
                .style(Style::default().fg(Theme::WARNING));
            frame.render_widget(para, area);
        }
    }
}
