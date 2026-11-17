mod agent;
mod app;
mod branding;
mod chat;
mod config;
mod focus_mode;
mod hardware;
mod hf;
mod splash;
mod tuner;
mod ui;

use anyhow::Result;
use app::{App, Screen, SettingField};
use branding::detect_utf8_support;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use splash::SPLASH_TOTAL_MS;
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Parser)]
#[command(name = "llmLab", version, about = "Ultra-lightweight local AI Agent & TUI runtime")]
struct CliArgs {
    /// Skip splash animation and open main UI immediately
    #[arg(long)]
    fast: bool,

    /// Open directly into AI Focus Mode (RAM & Window Cleaner)
    #[arg(long)]
    focus: bool,

    /// Open directly into Hugging Face Model Hub
    #[arg(long)]
    models: bool,

    /// Download latest llmLab.exe from GitHub Releases and replace old binary
    #[arg(long)]
    update: bool,

    /// Force ASCII logo fallback for legacy consoles
    #[arg(long)]
    ascii: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    if args.update {
        return run_self_update_cli();
    }

    let use_utf8 = detect_utf8_support(args.ascii);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let res = run_event_loop(&mut terminal, &args, use_utf8).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_self_update_cli() -> Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED: u32 = 0x0000_0008;
        println!("Запуск обновления llmLab (удаление старой версии и загрузка новой)...");
        let cmd = "irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex";
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", cmd])
            .creation_flags(DETACHED)
            .spawn();
    }
    #[cfg(not(windows))]
    {
        println!("Самообновление через PowerShell доступно на Windows.");
    }
    Ok(())
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    args: &CliArgs,
    use_utf8: bool,
) -> Result<()> {
    let (stream_tx, mut stream_rx) = mpsc::unbounded_channel();
    let (hf_tx, mut hf_rx) = mpsc::unbounded_channel();
    let (key_tx, mut key_rx) = mpsc::unbounded_channel();

    std::thread::spawn(move || loop {
        if event::poll(Duration::from_millis(50)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press && key_tx.send(key).is_err() {
                    break;
                }
            }
        }
    });

    let mut app = App::new(args.fast, args.focus, use_utf8, stream_tx, hf_tx);
    if args.models {
        app.screen = Screen::Models;
    }

    let mut tick = tokio::time::interval(Duration::from_millis(16)); // ~60 FPS
    let mut last_hw_refresh = Instant::now();

    while !app.should_quit {
        terminal.draw(|f| ui::render_app(f, &app))?;

        if app.screen == Screen::Splash
            && app.splash_started.elapsed().as_millis() as u64 >= SPLASH_TOTAL_MS
        {
            app.screen = if args.focus {
                Screen::FocusMode
            } else if args.models {
                Screen::Models
            } else {
                Screen::Dashboard
            };
        }

        tokio::select! {
            _ = tick.tick() => {
                if last_hw_refresh.elapsed() >= Duration::from_secs(1) {
                    app.refresh_telemetry();
                    last_hw_refresh = Instant::now();
                }
            }
            Some(key) = key_rx.recv() => {
                handle_key_event(&mut app, key, args.focus);
            }
            Some(stream_ev) = stream_rx.recv() => {
                app.handle_stream_event(stream_ev);
            }
            Some(hf_ev) = hf_rx.recv() => {
                app.handle_hf_event(hf_ev);
            }
        }
    }

    app.stop_engine();
    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent, start_in_focus: bool) {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => {
                app.request_quit();
                return;
            }
            KeyCode::Char('r') => {
                let count = app.focus.restore_closed_apps();
                app.status_banner = format!("Восстановлено приложений: {count}");
                return;
            }
            _ => {}
        }
    }

    if app.show_exit_restore_modal {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.focus.restore_closed_apps();
                app.stop_engine();
                app.should_quit = true;
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                app.stop_engine();
                app.should_quit = true;
            }
            KeyCode::Esc => {
                app.show_exit_restore_modal = false;
            }
            _ => {}
        }
        return;
    }

    if app.screen == Screen::Splash {
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc) {
            app.screen = if start_in_focus {
                Screen::FocusMode
            } else {
                Screen::Dashboard
            };
        }
        return;
    }

    // Text input capture inside HF search box or Chat prompt box
    let is_typing_hf = app.screen == Screen::Models && app.hf_search_focused;
    let is_typing_chat = app.screen == Screen::Chat && !app.chat_input.is_empty();

    if !is_typing_hf && !is_typing_chat {
        match key.code {
            KeyCode::Left => {
                app.screen = app.screen.prev_tab();
                return;
            }
            KeyCode::Right | KeyCode::Tab => {
                app.screen = app.screen.next_tab();
                return;
            }
            KeyCode::F(1) => {
                app.screen = Screen::Dashboard;
                return;
            }
            KeyCode::F(2) => {
                app.screen = Screen::Models;
                return;
            }
            KeyCode::F(3) => {
                app.screen = Screen::Chat;
                return;
            }
            KeyCode::F(4) => {
                app.screen = Screen::Skills;
                return;
            }
            KeyCode::F(5) => {
                app.focus.refresh();
                app.screen = Screen::FocusMode;
                return;
            }
            KeyCode::F(6) => {
                app.screen = Screen::Settings;
                return;
            }
            _ => {}
        }
    } else if key.code == KeyCode::Tab {
        app.hf_search_focused = false;
        app.screen = app.screen.next_tab();
        return;
    }

    match app.screen {
        Screen::Splash => {}
        Screen::Dashboard => match key.code {
            KeyCode::Char('1') | KeyCode::Enter => app.start_engine_and_open_chat(),
            KeyCode::Char('2') => app.screen = Screen::Models,
            KeyCode::Char('3') => app.screen = Screen::Skills,
            KeyCode::Char('4') => {
                app.focus.refresh();
                app.screen = Screen::FocusMode;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => app.refresh_models(),
            KeyCode::Char('u') | KeyCode::Char('U') => {
                let _ = run_self_update_cli();
                app.should_quit = true;
            }
            KeyCode::Up => {
                app.selected_model_idx = app.selected_model_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                if app.selected_model_idx + 1 < app.models.len() {
                    app.selected_model_idx += 1;
                }
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => app.request_quit(),
            _ => {}
        },
        Screen::Models => {
            if app.hf_search_focused {
                match key.code {
                    KeyCode::Esc => app.hf_search_focused = false,
                    KeyCode::Enter => {
                        app.hf_search_focused = false;
                        app.trigger_hf_search();
                    }
                    KeyCode::Backspace => {
                        app.hf_query.pop();
                    }
                    KeyCode::Char(c) => app.hf_query.push(c),
                    _ => {}
                }
                return;
            }

            match key.code {
                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S') => {
                    app.hf_search_focused = true;
                }
                KeyCode::Char('p') | KeyCode::Char('P') => app.trigger_hw_auto_pick(),
                KeyCode::Char('f') | KeyCode::Char('F') => app.inspect_hf_repo_files(),
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete => {
                    app.delete_selected_local_model();
                }
                KeyCode::Esc => {
                    if app.hf_in_file_picker {
                        app.hf_in_file_picker = false;
                    } else {
                        app.screen = Screen::Dashboard;
                    }
                }
                KeyCode::Up => {
                    if app.hf_in_file_picker {
                        app.hf_file_idx = app.hf_file_idx.saturating_sub(1);
                    } else {
                        app.hf_selected_idx = app.hf_selected_idx.saturating_sub(1);
                    }
                }
                KeyCode::Down => {
                    if app.hf_in_file_picker {
                        if app.hf_file_idx + 1 < app.hf_repo_files.len() {
                            app.hf_file_idx += 1;
                        }
                    } else if app.hf_selected_idx + 1 < app.hf_candidates.len() {
                        app.hf_selected_idx += 1;
                    }
                }
                KeyCode::Enter => app.open_or_download_hf_item(),
                _ => {}
            }
        }
        Screen::Chat => match key.code {
            KeyCode::Esc => {
                app.stop_engine();
                app.screen = Screen::Dashboard;
            }
            KeyCode::Enter => app.send_chat_prompt(),
            KeyCode::Backspace => {
                app.chat_input.pop();
            }
            KeyCode::PageUp => {
                app.chat_scroll = app.chat_scroll.saturating_sub(4);
            }
            KeyCode::PageDown => {
                app.chat_scroll = app.chat_scroll.saturating_add(4);
            }
            KeyCode::Char(c) => app.chat_input.push(c),
            _ => {}
        },
        Screen::Skills => match key.code {
            KeyCode::Up => {
                app.skill_cursor = app.skill_cursor.saturating_sub(1);
            }
            KeyCode::Down => {
                if app.skill_cursor < 4 {
                    app.skill_cursor += 1;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => app.toggle_skill_at_cursor(),
            KeyCode::Esc => app.screen = Screen::Dashboard,
            _ => {}
        },
        Screen::FocusMode => match key.code {
            KeyCode::Up => {
                app.focus.selected_idx = app.focus.selected_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                if app.focus.selected_idx + 1 < app.focus.targets.len() {
                    app.focus.selected_idx += 1;
                }
            }
            KeyCode::Char(' ') => app.focus.toggle_selected(),
            KeyCode::Char('a') | KeyCode::Char('A') => app.focus.toggle_all(),
            KeyCode::Char('r') | KeyCode::Char('R') => app.focus.refresh(),
            KeyCode::Enter => app.focus.apply_cleanup(),
            KeyCode::Char('f') | KeyCode::Char('F') => app.focus.force_kill_pending(),
            KeyCode::Esc => app.screen = Screen::Dashboard,
            _ => {}
        },
        Screen::Settings => match key.code {
            KeyCode::Up => {
                app.setting_idx = app.setting_idx.saturating_sub(1);
            }
            KeyCode::Down => {
                if app.setting_idx + 1 < SettingField::ALL.len() {
                    app.setting_idx += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => app.cycle_selected_setting(),
            KeyCode::Esc => app.screen = Screen::Dashboard,
            _ => {}
        },
    }
}
