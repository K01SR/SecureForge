use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Style, Stylize},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap}, text::{Line, Span}};
use crate::{app::App, theme::Theme};

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(0)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // Config panel
    let c = &app.carver;
    let cursor_marker = |idx: usize| if c.cursor_field == idx { ">" } else { " " };

    let field_style = |idx: usize| if c.cursor_field == idx {
        Style::default().fg(Theme::ACCENT).bold()
    } else {
        Style::default().fg(Theme::TEXT)
    };

    let config_lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!(" {} Source: ", cursor_marker(0)), field_style(0)),
            Span::styled(if c.source.is_empty() { "[enter path or /dev/sdX]".to_string() } else { c.source.clone() }, Style::default().fg(if c.source.is_empty() { Theme::MUTED } else { Theme::ACCENT })),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!(" {} Output: ", cursor_marker(1)), field_style(1)),
            Span::styled(c.output_dir.clone(), Style::default().fg(Theme::TEXT)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(format!(" {} Min Confidence: ", cursor_marker(2)), field_style(2)),
            Span::styled(format!("{}%", c.min_confidence), Style::default().fg(Theme::ACCENT2).bold()),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(Theme::SUCCESS).bold()),
            Span::styled(if c.scanning { "Scanning…" } else { "→ Start Scan" }, Style::default().fg(Theme::MUTED)),
        ]),
    ];

    let config_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if c.scanning { Theme::WARNING } else { Theme::BORDER }))
        .title(Span::styled(" ⬡ Forensic Carver Config ", Style::default().fg(Theme::SUCCESS).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    let config_para = Paragraph::new(config_lines).block(config_block).wrap(Wrap { trim: false });
    frame.render_widget(config_para, top[0]);

    // Progress panel
    let progress_inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2), Constraint::Min(0)])
        .margin(1)
        .split(top[1]);

    let progress_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Scan Progress ", Style::default().fg(Theme::SUCCESS).bold()))
        .style(Style::default().bg(Theme::SURFACE));
    frame.render_widget(progress_block, top[1]);

    let pct = (c.progress * 100.0) as u16;
    let status = if c.scanning { format!("Scanning… {:.1}%", c.progress * 100.0) } else if pct == 100 { format!("Complete — {} files recovered", c.found.len()) } else { "Ready".to_string() };
    let status_para = Paragraph::new(status).style(Style::default().fg(Theme::TEXT));
    frame.render_widget(status_para, progress_inner[0]);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Theme::SUCCESS).bg(Theme::BORDER))
        .percent(pct)
        .label(format!("{}%", pct));
    frame.render_widget(gauge, progress_inner[1]);

    // Results list
    let result_items: Vec<ListItem> = c.found.iter().map(|f| {
        ListItem::new(Line::from(vec![
            Span::styled(" ✓ ", Style::default().fg(Theme::SUCCESS)),
            Span::styled(f, Style::default().fg(Theme::TEXT)),
        ]))
    }).chain(c.log.iter().map(|l| {
        ListItem::new(Line::from(Span::styled(format!(" › {}", l), Style::default().fg(Theme::MUTED))))
    })).collect();

    let results_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
        .title(Span::styled(" Recovered Files ", Style::default().fg(Theme::SUCCESS).bold()))
        .style(Style::default().bg(Theme::BG));
    let results_list = List::new(result_items).block(results_block);
    frame.render_widget(results_list, chunks[1]);
}
