use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod theme;
mod ui;
mod worker;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = Arc::new(Mutex::new(app::App::new()));
    {
        let mut a = app.lock().unwrap();
        a.load_drives();
        a.push_log("SecureForge TUI started".to_string());
        let count = a.drives.len();
        a.push_log(format!("Detected {} drives", count));
    }

    let tick_rate = Duration::from_millis(60);

    loop {
        // Draw
        {
            let mut a = app.lock().unwrap();
            terminal.draw(|f| ui::render(f, &mut a))?;
        }

        // Poll events
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                let needs_wipe = {
                    let mut a = app.lock().unwrap();
                    handle_key_inner(&mut a, key.code, key.modifiers)
                };
                if let Some((path, method, verify)) = needs_wipe {
                    {
                        let mut a = app.lock().unwrap();
                        a.wipe_phase = app::WipePhase::Running {
                            pass: 1, total_passes: 1, bytes_done: 0, bytes_total: 1,
                            speed_mbps: 0.0, started: std::time::Instant::now(),
                        };
                    }
                    worker::start_wipe(Arc::clone(&app), path, method, verify);
                }
            }
        } else {
            let mut a = app.lock().unwrap();
            a.tick = a.tick.wrapping_add(1);
        }

        let should_quit = app.lock().unwrap().should_quit;
        if should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn handle_key_inner(app: &mut app::App, code: KeyCode, modifiers: KeyModifiers) -> Option<(String, app::WipeMethod, bool)> {
    // Global quit
    if code == KeyCode::Char('q') || code == KeyCode::Char('Q')
        || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
    {
        app.should_quit = true;
        return None;
    }

    // Popup handling
    if app.popup != app::Popup::None {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let app::Popup::Confirm { .. } = &app.popup {
                    let drive_path = app.selected_drive
                        .and_then(|i| app.drives.get(i))
                        .map(|d| d.path.clone());
                    let method = app.selected_method();
                    let do_verify = app.wipe_verify;
                    app.popup = app::Popup::None;
                    if let Some(path) = drive_path {
                        return Some((path, method, do_verify));
                    }
                } else {
                    app.popup = app::Popup::None;
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.popup = app::Popup::None;
                app.wipe_phase = app::WipePhase::Idle;
            }
            _ => {}
        }
        return None;
    }

    // Function keys for screen navigation
    match code {
        KeyCode::F(1) => { app.screen = app::Screen::Dashboard; }
        KeyCode::F(2) => { app.screen = app::Screen::DriveManager; }
        KeyCode::F(3) => { app.screen = app::Screen::WipeWizard; }
        KeyCode::F(4) => { app.screen = app::Screen::Carver; }
        KeyCode::F(5) => { app.screen = app::Screen::Help; }
        _ => {}
    }

    // Screen-specific key handling
    match app.screen {
        app::Screen::DriveManager => handle_drives(app, code),
        app::Screen::WipeWizard => handle_wipe(app, code),
        app::Screen::Carver => handle_carver(app, code),
        _ => {}
    }
    
    None
}

fn handle_drives(app: &mut app::App, code: KeyCode) {
    match code {
        KeyCode::Up => { if app.drive_cursor > 0 { app.drive_cursor -= 1; } }
        KeyCode::Down => { if app.drive_cursor + 1 < app.drives.len() { app.drive_cursor += 1; } }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.load_drives();
            app.set_status("Drive list refreshed");
        }
        KeyCode::Enter => {
            if !app.drives.is_empty() {
                app.selected_drive = Some(app.drive_cursor);
                let name = app.drives[app.drive_cursor].name.clone();
                app.set_status(format!("Selected: /dev/{}", name));
                app.screen = app::Screen::WipeWizard;
            }
        }
        _ => {}
    }
}

fn handle_wipe(app: &mut app::App, code: KeyCode) {
    // Don't accept new commands while wipe is running
    if matches!(app.wipe_phase, app::WipePhase::Running { .. } | app::WipePhase::Verifying) {
        return;
    }

    match code {
        KeyCode::Up => {
            if app.wipe_method_cursor > 0 { app.wipe_method_cursor -= 1; }
        }
        KeyCode::Down => {
            let max = app::WipeMethod::all().len() - 1;
            if app.wipe_method_cursor < max { app.wipe_method_cursor += 1; }
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            app.wipe_verify = !app.wipe_verify;
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.wipe_expert = !app.wipe_expert;
        }
        KeyCode::Enter => {
            if app.selected_drive.is_none() {
                app.set_status("⚠ No drive selected — go to F2 Drives");
                return;
            }
            let d = &app.drives[app.selected_drive.unwrap()];
            if d.is_system && !app.wipe_expert {
                app.popup = app::Popup::Error(
                    "This is a SYSTEM drive. Enable Expert mode (E) first.".to_string()
                );
                return;
            }
            app.popup = app::Popup::Confirm {
                title: "⚠ Confirm Permanent Erasure".to_string(),
                message: format!(
                    "About to PERMANENTLY erase /dev/{} ({}) using {}. This is irreversible.",
                    d.name, d.model, app.selected_method().label()
                ),
            };
            app.wipe_phase = app::WipePhase::Confirming;
        }
        _ => {}
    }
}

fn handle_carver(app: &mut app::App, code: KeyCode) {
    if app.carver.scanning { return; }
    match code {
        KeyCode::Tab => {
            app.carver.cursor_field = (app.carver.cursor_field + 1) % 3;
        }
        KeyCode::Up => {
            if app.carver.cursor_field == 2 && app.carver.min_confidence < 100 {
                app.carver.min_confidence += 5;
            }
        }
        KeyCode::Down => {
            if app.carver.cursor_field == 2 && app.carver.min_confidence > 5 {
                app.carver.min_confidence -= 5;
            }
        }
        KeyCode::Char(c) => {
            match app.carver.cursor_field {
                0 => app.carver.source.push(c),
                1 => app.carver.output_dir.push(c),
                _ => {}
            }
        }
        KeyCode::Backspace => {
            match app.carver.cursor_field {
                0 => { app.carver.source.pop(); }
                1 => { app.carver.output_dir.pop(); }
                _ => {}
            }
        }
        KeyCode::Enter => {
            if app.carver.source.is_empty() {
                app.set_status("⚠ Enter source path or device");
                return;
            }
            app.carver.scanning = true;
            app.carver.found.clear();
            app.carver.log.clear();
            app.carver.progress = 0.0;
            // NOTE: real carver implementation uses sih149_core scanner in a thread;
            // for now simulate progress to keep UI responsive:
            let source = app.carver.source.clone();
            app.carver.log.push(format!("Starting scan of {}", source));
            app.set_status(format!("Scanning {}", source));
            // In production: spawn thread calling sih149_core::carver and updating app.carver
        }
        _ => {}
    }
}
