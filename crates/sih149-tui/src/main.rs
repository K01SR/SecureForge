#![allow(dead_code, unused_imports, unused_mut)]
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

#[cfg(test)]
mod tests;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = Arc::new(Mutex::new(app::App::new()));
    {
        let mut a = app.lock().unwrap();
        a.load_drives();
        a.push_log_level(app::LogLevel::Info, "SecureForge TUI started");
        let n = a.drives.len();
        a.push_log_level(
            if n > 0 { app::LogLevel::Success } else { app::LogLevel::Warning },
            format!("{} block device(s) detected", n),
        );
        if n == 0 {
            a.push_log_level(app::LogLevel::Warning, "Try running with: sudo sforge");
        }
    }

    let tick_rate = Duration::from_millis(50); // 20fps

    loop {
        {
            let mut a = app.lock().unwrap();
            terminal.draw(|f| ui::render(f, &mut a))?;
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // Ignore key-release events (crossterm sends both on some terminals)
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }
                let needs_wipe = {
                    let mut a = app.lock().unwrap();
                    handle_key(&mut a, key.code, key.modifiers)
                };
                if let Some((path, method, verify)) = needs_wipe {
                    {
                        let mut a = app.lock().unwrap();
                        a.wipe_phase = app::WipePhase::Running {
                            pass: 1,
                            total_passes: method.passes(),
                            bytes_done: 0,
                            bytes_total: 1,
                            speed_mbps: 0.0,
                            started: std::time::Instant::now(),
                        };
                        a.push_log_level(app::LogLevel::Warning, format!("Wipe started: {} → {}", path, method.label()));
                    }
                    worker::start_wipe(Arc::clone(&app), path, method, verify);
                }
            }
        } else {
            let mut a = app.lock().unwrap();
            a.tick = a.tick.wrapping_add(1);
        }

        if app.lock().unwrap().should_quit { break; }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Returns Some((path, method, verify)) when a wipe should be started in a background thread.
fn handle_key(app: &mut app::App, code: KeyCode, modifiers: KeyModifiers) -> Option<(String, app::WipeMethod, bool)> {
    // Global: quit
    if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q'))
        || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
    {
        app.should_quit = true;
        return None;
    }

    // Popup handling takes priority
    if app.popup != app::Popup::None {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                if let app::Popup::Confirm { .. } = &app.popup.clone() {
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
                if matches!(app.wipe_phase, app::WipePhase::Confirming) {
                    app.wipe_phase = app::WipePhase::Idle;
                }
            }
            _ => {}
        }
        return None;
    }

    // Global screen navigation via function keys
    match code {
        KeyCode::F(1) => { app.navigate(app::Screen::Dashboard); return None; }
        KeyCode::F(2) => { app.navigate(app::Screen::DriveManager); return None; }
        KeyCode::F(3) => {
            app.entropy_drive_cursor = app.drive_cursor;
            app.navigate(app::Screen::Entropy);
            return None;
        }
        KeyCode::F(4) => { app.navigate(app::Screen::WipeWizard); return None; }
        KeyCode::F(5) => { app.navigate(app::Screen::Carver); return None; }
        KeyCode::F(6) => { app.navigate(app::Screen::Help); return None; }
        KeyCode::Esc  => {
            // Go back to previous screen
            let prev = app.prev_screen.clone();
            app.navigate(prev);
            return None;
        }
        _ => {}
    }

    // Screen-specific handlers
    match app.screen.clone() {
        app::Screen::Dashboard    => { /* no special keys */ }
        app::Screen::DriveManager => handle_drives(app, code),
        app::Screen::Entropy      => handle_entropy(app, code),
        app::Screen::WipeWizard   => handle_wipe(app, code),
        app::Screen::Carver       => handle_carver(app, code),
        app::Screen::Help         => {}
    }

    None
}

fn handle_drives(app: &mut app::App, code: KeyCode) {
    match code {
        KeyCode::Up => {
            if app.drive_cursor > 0 { app.drive_cursor -= 1; }
        }
        KeyCode::Down => {
            if app.drive_cursor + 1 < app.drives.len() { app.drive_cursor += 1; }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.load_drives();
            app.set_status("Drive list refreshed");
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.entropy_drive_cursor = app.drive_cursor;
            app.navigate(app::Screen::Entropy);
            // Immediately trigger entropy load
            let idx = app.entropy_drive_cursor;
            app.sample_drive_entropy(idx, 256);
            app.set_status(format!("Entropy analysis complete for /dev/{}", app.drives.get(idx).map(|d| d.name.as_str()).unwrap_or("?")));
        }
        KeyCode::Enter => {
            if !app.drives.is_empty() {
                app.selected_drive = Some(app.drive_cursor);
                let name = app.drives[app.drive_cursor].name.clone();
                app.set_status(format!("Selected /dev/{} for sanitization", name));
                app.push_log_level(app::LogLevel::Info, format!("Drive selected: /dev/{}", name));
                app.navigate(app::Screen::WipeWizard);
            }
        }
        _ => {}
    }
}

fn handle_entropy(app: &mut app::App, code: KeyCode) {
    match code {
        KeyCode::Up => {
            if app.entropy_drive_cursor > 0 { app.entropy_drive_cursor -= 1; }
        }
        KeyCode::Down => {
            if app.entropy_drive_cursor + 1 < app.drives.len() { app.entropy_drive_cursor += 1; }
        }
        KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter => {
            let idx = app.entropy_drive_cursor;
            if idx < app.drives.len() {
                app.set_status("Analyzing entropy…");
                app.sample_drive_entropy(idx, 256);
                let name = app.drives[idx].name.clone();
                let avg = if !app.drives[idx].entropy_samples.is_empty() {
                    let s = &app.drives[idx].entropy_samples;
                    s.iter().sum::<f64>() / s.len() as f64
                } else { 0.0 };
                app.push_log_level(app::LogLevel::Success, format!("Entropy /dev/{}: {:.3} bits/byte avg", name, avg));
                app.set_status(format!("Entropy loaded — /dev/{}: {:.3} bits/byte", name, avg));
            }
        }
        _ => {}
    }
}

fn handle_wipe(app: &mut app::App, code: KeyCode) {
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
            app.set_status(format!("Post-wipe verify: {}", if app.wipe_verify { "ON" } else { "OFF" }));
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.wipe_expert = !app.wipe_expert;
            app.set_status(if app.wipe_expert { "Expert mode enabled" } else { "Expert mode disabled" });
        }
        KeyCode::Enter => {
            if app.selected_drive.is_none() {
                app.set_error("No drive selected — press F2, select a drive, then Enter");
                return;
            }
            let d = &app.drives[app.selected_drive.unwrap()];
            if d.is_system && !app.wipe_expert {
                app.popup = app::Popup::Error(
                    "This is a SYSTEM / BOOT drive. Toggle Expert mode with [E] first to confirm you know what you're doing.".to_string(),
                );
                return;
            }
            app.popup = app::Popup::Confirm {
                title: "⚠ Confirm Permanent Erasure".to_string(),
                message: format!(
                    "PERMANENTLY erase /dev/{} ({}, {}) using {}?\n\n  This will DESTROY ALL DATA and cannot be undone.",
                    d.name,
                    d.model.chars().take(24).collect::<String>(),
                    super_format_bytes(d.size_bytes),
                    app.selected_method().label(),
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
            if app.carver.cursor_field == 2 {
                if app.carver.min_confidence < 100 { app.carver.min_confidence = app.carver.min_confidence.saturating_add(5).min(100); }
            } else if !app.carver.found.is_empty() && app.carver.result_cursor > 0 {
                app.carver.result_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if app.carver.cursor_field == 2 {
                if app.carver.min_confidence > 5 { app.carver.min_confidence = app.carver.min_confidence.saturating_sub(5).max(5); }
            } else if !app.carver.found.is_empty() && app.carver.result_cursor + 1 < app.carver.found.len() {
                app.carver.result_cursor += 1;
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
                app.set_error("Source path is empty — type a path or /dev/sdX");
                return;
            }
            app.carver.scanning = true;
            app.carver.found.clear();
            app.carver.log.clear();
            app.carver.progress = 0.0;
            app.carver.result_cursor = 0;
            let src = app.carver.source.clone();
            app.carver.log.push(format!("Starting scan of {}", src));
            app.push_log_level(app::LogLevel::Info, format!("Carver scan started: {}", src));
            app.set_status(format!("Scanning {}", src));
            // Real: spawn worker thread calling sih149_core::carver scanner
        }
        _ => {}
    }
}

fn super_format_bytes(n: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = n as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < units.len() - 1 { size /= 1024.0; idx += 1; }
    format!("{:.1} {}", size, units[idx])
}
