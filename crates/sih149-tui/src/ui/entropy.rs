/// Full-screen entropy heatmap analyzer with multi-color block visualization,
/// per-bucket stats, legend, and drive selector.
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Gauge},
    text::{Line, Span},
};
use crate::{app::{App, entropy_label}, theme::Theme};
use super::format_bytes;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(area);

    render_drive_list(frame, main[0], app);
    render_entropy_panel(frame, main[1], app);
}

fn render_drive_list(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.drives.iter().enumerate().map(|(i, d)| {
        let selected = app.entropy_drive_cursor == i;

        let (avg_e, avg_str) = if d.entropy_loaded && !d.entropy_samples.is_empty() {
            let avg = d.entropy_samples.iter().sum::<f64>() / d.entropy_samples.len() as f64;
            (avg, format!("{:.2}b", avg))
        } else {
            (0.0, "  ─   ".to_string())
        };

        let ind = if selected { "▶" } else { " " };
        let name_color = if selected { Theme::CYAN } else { Theme::TEXT };
        let bg = if selected { Theme::SURFACE2 } else { Theme::BG };
        let ent_color = if d.entropy_loaded { Theme::entropy_color(avg_e) } else { Theme::MUTED };

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(format!(" {} ", ind), Style::default().fg(Theme::CYAN)),
                Span::styled(format!("/dev/{:<8}", d.name), Style::default().fg(name_color).add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() })),
            ]),
            Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(format!("{:<6}", d.drive_type), Style::default().fg(Theme::BLUE)),
                Span::styled(format!("  {}", avg_str), Style::default().fg(ent_color)),
            ]),
            Line::raw(""),
        ]).style(Style::default().bg(bg))
    }).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Drives [↑↓] [E] Load ",
            Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Theme::BG));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_entropy_panel(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),   // header / info
            Constraint::Length(4),   // legend
            Constraint::Min(0),      // heatmap
            Constraint::Length(6),   // stats bar
        ])
        .split(area);

    let d = match app.drives.get(app.entropy_drive_cursor) {
        Some(d) => d,
        None => {
            let para = Paragraph::new("  No drives available.")
                .style(Style::default().fg(Theme::MUTED));
            frame.render_widget(para, area);
            return;
        }
    };

    // ── Header ────────────────────────────────────────────────────────────────
    let (avg_e, avg_label) = if d.entropy_loaded && !d.entropy_samples.is_empty() {
        let avg = d.entropy_samples.iter().sum::<f64>() / d.entropy_samples.len() as f64;
        let (lbl, _) = entropy_label(avg);
        (avg, lbl)
    } else {
        (0.0, "Not analyzed")
    };

    let header_lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  /dev/", Style::default().fg(Theme::MUTED)),
            Span::styled(&d.name, Style::default().fg(Theme::CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("  —  ", Style::default().fg(Theme::MUTED)),
            Span::styled(&d.model, Style::default().fg(Theme::TEXT)),
            Span::styled("  ", Style::default()),
            Span::styled(&d.drive_type, Style::default().fg(Theme::BLUE)),
            Span::styled("  ", Style::default()),
            Span::styled(format_bytes(d.size_bytes), Style::default().fg(Theme::TEXT_DIM)),
        ]),
        Line::raw(""),
        if d.entropy_loaded {
            Line::from(vec![
                Span::styled("  Average Entropy: ", Style::default().fg(Theme::MUTED)),
                Span::styled(format!("{:.3} bits/byte", avg_e), Style::default().fg(Theme::entropy_color(avg_e)).add_modifier(Modifier::BOLD)),
                Span::styled("  →  ", Style::default().fg(Theme::MUTED)),
                Span::styled(avg_label, Style::default().fg(Theme::entropy_color(avg_e)).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  ({} samples)", d.entropy_samples.len()), Style::default().fg(Theme::MUTED)),
            ])
        } else {
            Line::from(Span::styled("  Press E to analyze entropy of this drive", Style::default().fg(Theme::WARNING)))
        },
    ];

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" ▓ Entropy Analysis ", Style::default().fg(Theme::PURPLE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Theme::SURFACE));
    let header_para = Paragraph::new(header_lines).block(header_block);
    frame.render_widget(header_para, rows[0]);

    // ── Legend ────────────────────────────────────────────────────────────────
    render_legend(frame, rows[1]);

    // ── Heatmap ───────────────────────────────────────────────────────────────
    render_heatmap(frame, rows[2], d);

    // ── Per-bucket stats ──────────────────────────────────────────────────────
    render_stats(frame, rows[3], d, avg_e);
}

fn render_legend(frame: &mut Frame, area: Rect) {
    let bands: &[(&str, ratatui::style::Color, &str)] = &[
        ("░ 0–1b", Theme::ENT_0, "Dead/Zero"),
        ("░ 1–2b", Theme::ENT_1, "Near-zero"),
        ("▒ 2–4b", Theme::ENT_2, "Low"),
        ("▒ 4–6b", Theme::ENT_3, "Moderate"),
        ("▓ 6–7b", Theme::ENT_4, "High"),
        ("█ 7–8b", Theme::ENT_5, "Encrypted"),
    ];

    let mut spans = vec![Span::raw("  ")];
    for (sym, color, label) in bands {
        spans.push(Span::styled(*sym, Style::default().fg(*color).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {} ", label), Style::default().fg(Theme::MUTED)));
        spans.push(Span::styled("  ", Style::default()));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Entropy Legend ", Style::default().fg(Theme::TEXT_DIM)))
        .style(Style::default().bg(Theme::BG));
    let para = Paragraph::new(vec![Line::raw(""), Line::from(spans)]).block(block);
    frame.render_widget(para, area);
}

fn render_heatmap(frame: &mut Frame, area: Rect, d: &crate::app::DriveEntry) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(
            " Entropy Heatmap — each column = one sector sample ",
            Style::default().fg(Theme::CYAN),
        ))
        .style(Style::default().bg(Theme::BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if !d.entropy_loaded || d.entropy_samples.is_empty() {
        let para = Paragraph::new(vec![
            Line::raw(""),
            Line::from(Span::styled(
                "  No entropy data. Select drive with ↑↓ and press E to analyze.",
                Style::default().fg(Theme::MUTED),
            )),
        ]);
        frame.render_widget(para, inner);
        return;
    }

    let samples = &d.entropy_samples;
    let w = inner.width as usize;
    let h = inner.height as usize;

    // Fill every row of the heatmap area for density
    let step = (samples.len() as f64 / w as f64).max(1.0);
    let mut lines: Vec<Line> = Vec::with_capacity(h);

    for _row in 0..h {
        let mut spans = Vec::with_capacity(w + 2);
        spans.push(Span::raw(" "));
        for col in 0..w {
            let idx = (col as f64 * step) as usize;
            let e = samples.get(idx).copied().unwrap_or(0.0);
            let color = Theme::entropy_color(e);

            // Vary character by entropy magnitude for visual depth
            let ch = if e < 1.0 { '░' }
                else if e < 3.0 { '▒' }
                else if e < 6.0 { '▓' }
                else { '█' };

            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        lines.push(Line::from(spans));
    }

    // Overlay offset labels every 20% along bottom row
    if h > 2 {
        let positions = [0, w/5, 2*w/5, 3*w/5, 4*w/5, w.saturating_sub(4)];
        let total_gb = d.size_bytes as f64 / 1e9;
        let label_line_spans = vec![Span::raw(" ")];
        let last_line = lines.last_mut().unwrap();
        let _ = last_line; // we just rebuild it

        let mut bottom = vec![Span::raw(" ")];
        for col in 0..w {
            let idx = (col as f64 * step) as usize;
            let e = samples.get(idx).copied().unwrap_or(0.0);
            let color = Theme::entropy_color(e);
            let pos_pct = col as f64 / w as f64;

            // Check if this column should show a label
            let label_idx = positions.iter().position(|&p| (col as isize - p as isize).abs() < 3);
            if let Some(li) = label_idx {
                let gb = total_gb * positions[li] as f64 / w as f64;
                let lbl = format!("{:.1}G", gb);
                bottom.push(Span::styled(lbl, Style::default().fg(Theme::MUTED)));
                let _ = pos_pct;
            } else {
                let ch = if e < 1.0 { '░' } else if e < 3.0 { '▒' } else if e < 6.0 { '▓' } else { '█' };
                bottom.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }
            let _ = label_line_spans;
        }
        if let Some(last) = lines.last_mut() {
            *last = Line::from(bottom);
        }
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_stats(frame: &mut Frame, area: Rect, d: &crate::app::DriveEntry, avg_e: f64) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Distribution Statistics ", Style::default().fg(Theme::TEXT_DIM)))
        .style(Style::default().bg(Theme::SURFACE));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if !d.entropy_loaded || d.entropy_samples.is_empty() { return; }

    let samples = &d.entropy_samples;
    let n = samples.len() as f64;

    // Bucket counts
    let buckets = [
        ("Dead/Zero  0–1b", 0.0_f64..1.0_f64, Theme::ENT_0),
        ("Near-zero  1–2b", 1.0..2.0, Theme::ENT_1),
        ("Low        2–4b", 2.0..4.0, Theme::ENT_2),
        ("Moderate   4–6b", 4.0..6.0, Theme::ENT_3),
        ("High       6–7b", 6.0..7.0, Theme::ENT_4),
        ("Encrypted  7–8b", 7.0..8.01, Theme::ENT_5),
    ];

    let row_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); 4])
        .split(inner);

    // Two-column layout
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(if !row_layout.is_empty() { row_layout[0] } else { inner });

    // Summary line
    let (min_e, max_e) = samples.iter().fold((f64::MAX, f64::MIN), |(mn, mx), &e| (mn.min(e), mx.max(e)));
    let summary = Line::from(vec![
        Span::styled("  min ", Style::default().fg(Theme::MUTED)),
        Span::styled(format!("{:.2}b", min_e), Style::default().fg(Theme::entropy_color(min_e)).add_modifier(Modifier::BOLD)),
        Span::styled("  avg ", Style::default().fg(Theme::MUTED)),
        Span::styled(format!("{:.2}b", avg_e), Style::default().fg(Theme::entropy_color(avg_e)).add_modifier(Modifier::BOLD)),
        Span::styled("  max ", Style::default().fg(Theme::MUTED)),
        Span::styled(format!("{:.2}b", max_e), Style::default().fg(Theme::entropy_color(max_e)).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  │  {} samples total", samples.len()), Style::default().fg(Theme::MUTED)),
    ]);
    let _ = cols;
    let _ = row_layout;

    // Render gauge per bucket
    let gauge_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); buckets.len() + 1])
        .split(inner);

    frame.render_widget(Paragraph::new(summary), gauge_area[0]);

    for (i, (label, range, color)) in buckets.iter().enumerate() {
        let count = samples.iter().filter(|&&e| range.contains(&e)).count();
        let pct = ((count as f64 / n) * 100.0) as u16;
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(*color))
            .percent(pct)
            .label(format!("{:<20} {:>3}% ({} sectors)", label, pct, count));
        if let Some(&ga) = gauge_area.get(i + 1) {
            frame.render_widget(gauge, ga);
        }
    }
}
