pub mod dashboard;
pub mod drives;
pub mod wipe;
pub mod carver;
pub mod help;

use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Frame};
use crate::app::App;
use crate::theme::Theme;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::text::{Line, Span};

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Split into header, body, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    render_header(frame, chunks[0], app);

    match app.screen {
        crate::app::Screen::Dashboard => dashboard::render(frame, chunks[1], app),
        crate::app::Screen::DriveManager => drives::render(frame, chunks[1], app),
        crate::app::Screen::WipeWizard => wipe::render(frame, chunks[1], app),
        crate::app::Screen::Carver => carver::render(frame, chunks[1], app),
        crate::app::Screen::Help => help::render(frame, chunks[1], app),
    }

    render_statusbar(frame, chunks[2], app);

    // Overlay popup if any
    if app.popup != crate::app::Popup::None {
        render_popup(frame, area, app);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let tabs = vec![
        ("F1 Dashboard", crate::app::Screen::Dashboard),
        ("F2 Drives", crate::app::Screen::DriveManager),
        ("F3 Wipe", crate::app::Screen::WipeWizard),
        ("F4 Carver", crate::app::Screen::Carver),
        ("F5 Help", crate::app::Screen::Help),
    ];

    let spans: Vec<Span> = tabs.iter().enumerate().map(|(i, (label, screen))| {
        let active = &app.screen == screen;
        let sep = if i == 0 { "" } else { "  " };
        if active {
            Span::styled(format!("{} {} ", sep, label), Style::default().fg(Theme::BG).bg(Theme::ACCENT).bold())
        } else {
            Span::styled(format!("{} {} ", sep, label), Style::default().fg(Theme::MUTED).bg(Theme::SURFACE))
        }
    }).collect();

    let logo = Span::styled(" ⬡ SECUREFORGE ", Style::default().fg(Theme::ACCENT).bg(Theme::SURFACE).bold());
    let mut all_spans = vec![logo, Span::raw("  ")];
    all_spans.extend(spans);

    let line = Line::from(all_spans);
    let block = Block::default().style(Style::default().bg(Theme::SURFACE));
    let para = Paragraph::new(line).block(block);
    frame.render_widget(para, area);
}

fn render_statusbar(frame: &mut Frame, area: Rect, app: &App) {
    let msg = if let Some((ref m, ref t)) = app.status_msg {
        if t.elapsed().as_secs() < 5 { m.clone() } else { String::new() }
    } else { String::new() };

    let keys_hint = match app.screen {
        crate::app::Screen::DriveManager => "↑↓ Navigate  Enter Select  R Refresh  Q Quit",
        crate::app::Screen::WipeWizard => "↑↓ Method  Space Toggle  Enter Execute  Q Quit",
        crate::app::Screen::Carver => "Tab Fields  Enter Start  Q Quit",
        _ => "F1-F5 Navigate  Q Quit",
    };

    let left = if msg.is_empty() {
        Span::styled(format!(" {} ", keys_hint), Style::default().fg(Theme::MUTED).bg(Theme::SURFACE))
    } else {
        Span::styled(format!(" {} ", msg), Style::default().fg(Theme::SUCCESS).bg(Theme::SURFACE).bold())
    };

    let tick_char = ["||", "|\\", "--", "/|"][( app.tick / 4) as usize % 4];
    let right = Span::styled(format!(" {} NIST SP 800-88 R1 ", tick_char), Style::default().fg(Theme::ACCENT).bg(Theme::SURFACE));

    let line = Line::from(vec![left, Span::raw(""), right]);
    let para = Paragraph::new(line).style(Style::default().bg(Theme::SURFACE));
    frame.render_widget(para, area);
}

fn render_popup(frame: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(60, 30, area);
    use ratatui::widgets::Clear;
    frame.render_widget(Clear, popup_area);

    let (title, body, color) = match &app.popup {
        crate::app::Popup::Confirm { title, message } => (title.as_str(), message.as_str(), Theme::WARNING),
        crate::app::Popup::Error(e) => ("Error", e.as_str(), Theme::DANGER),
        crate::app::Popup::Info(i) => ("Info", i.as_str(), Theme::ACCENT),
        crate::app::Popup::None => return,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(format!(" {} ", title), Style::default().fg(color).bold()))
        .style(Style::default().bg(Theme::SURFACE));

    let buttons = "\n\n  [Y] Confirm    [N] Cancel";
    let text = format!("\n{}\n{}", body, buttons);
    let para = Paragraph::new(text).block(block).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(para, popup_area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
