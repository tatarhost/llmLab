use crate::app::{App, Screen, SettingField};
use crate::branding::{gradient_color, LOGO_ASCII, LOGO_TAGLINE, LOGO_UNICODE};
use crate::focus_mode::ProcessRowStatus;
use crate::hardware::format_gb;
use crate::tuner::optimal_threads;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table, TableState, Wrap,
};
use ratatui::Frame;

const CYAN: Color = Color::Rgb(0, 245, 212);
const BLUE: Color = Color::Rgb(58, 134, 255);
const BORDER_DIM: Color = Color::Rgb(42, 68, 98);

pub fn render_app(frame: &mut Frame<'_>, app: &App) {
    match app.screen {
        Screen::Splash => {
            crate::splash::render_splash(
                frame,
                app.splash_started.elapsed().as_millis() as u64,
                app.use_utf8,
                &app.hw,
            );
        }
        Screen::Dashboard => render_dashboard(frame, app),
        Screen::FocusMode => render_focus_mode(frame, app),
        Screen::Chat => render_chat(frame, app),
        Screen::Settings => render_settings(frame, app),
    }

    if app.show_exit_restore_modal {
        render_exit_restore_modal(frame, app);
    }
}

fn render_top_bar(frame: &mut Frame<'_>, area: Rect, app: &App, active_tab: &str) {
    let mem_pct = if app.hw.ram_total_bytes > 0 {
        (app.hw.ram_used_bytes as f64 / app.hw.ram_total_bytes as f64) * 100.0
    } else {
        0.0
    };
    let title_line = Line::from(vec![
        Span::styled(" llmLab ", Style::default().fg(Color::Black).bg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  [{active_tab}]  "), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(
                "RAM: {} / {} ({mem_pct:.0}%)  |  CPU: {:.0}%  |  P-Threads: {}",
                format_gb(app.hw.ram_used_bytes),
                format_gb(app.hw.ram_total_bytes),
                app.hw.cpu_usage_percent,
                optimal_threads(&app.hw.cpu),
            ),
            Style::default().fg(Color::Rgb(170, 210, 245)),
        ),
    ]);

    let bar = Paragraph::new(title_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_DIM)),
    );
    frame.render_widget(bar, area);
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, help: &str, status: &str) {
    let text = vec![
        Line::from(Span::styled(status, Style::default().fg(CYAN))),
        Line::from(Span::styled(help, Style::default().fg(Color::DarkGray))),
    ];
    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_DIM)),
    );
    frame.render_widget(p, area);
}

fn render_dashboard(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_top_bar(frame, chunks[0], app, "Главное меню");

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(46), Constraint::Percentage(54)])
        .split(chunks[1]);

    // Left panel: compact brand + hardware summary + quick navigation
    let left_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(8)])
        .split(body[0]);

    let logo = if app.use_utf8 { &LOGO_UNICODE } else { &LOGO_ASCII };
    let mut brand_lines: Vec<Line<'_>> = logo
        .iter()
        .enumerate()
        .map(|(idx, &l)| {
            let color = gradient_color(idx as f32 / 17.0);
            Line::from(Span::styled(l, Style::default().fg(color)))
        })
        .collect();
    brand_lines.push(Line::from(Span::styled(
        LOGO_TAGLINE,
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    let brand_widget = Paragraph::new(brand_lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_DIM))
                .title(" Runtime "),
        );
    frame.render_widget(brand_widget, left_rows[0]);

    let actions = vec![
        Line::from(vec![
            Span::styled("[1 / Enter] ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::raw("Запустить ИИ-чат (Auto-Tune)"),
        ]),
        Line::from(vec![
            Span::styled("[2]         ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::raw("Режим «Только ИИ» (очистка RAM + выбор окон)"),
        ]),
        Line::from(vec![
            Span::styled("[3]         ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::raw("Профили движка (AVX2 / Vulkan / OpenVINO / Cloud)"),
        ]),
        Line::from(vec![
            Span::styled("[Ctrl+R]    ", Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
            Span::raw("Восстановить закрытые окна"),
        ]),
        Line::from(vec![
            Span::styled("[Q]         ", Style::default().fg(Color::DarkGray)),
            Span::raw("Выход"),
        ]),
    ];
    let nav_block = Paragraph::new(actions).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_DIM))
            .title(" Действия "),
    );
    frame.render_widget(nav_block, left_rows[1]);

    // Right panel: hardware profile + local GGUF models
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(6)])
        .split(body[1]);

    let hw_lines = vec![
        Line::from(format!("CPU:    {}", app.hw.cpu.brand)),
        Line::from(format!(
            "Ядра:   {} P / {} E ({} логич.) · AVX2: {}",
            app.hw.cpu.p_cores,
            app.hw.cpu.e_cores,
            app.hw.cpu.logical_threads,
            if app.hw.cpu.avx2 { "Да" } else { "Нет" }
        )),
        Line::from(format!(
            "GPU:    {} ({}) · Vulkan: {} · OpenVINO: {}",
            app.hw.gpu.name,
            app.hw.gpu.vendor,
            if app.hw.gpu.vulkan_available { "Да" } else { "Нет" },
            if app.hw.gpu.openvino_available { "Да" } else { "Нет" }
        )),
        Line::from(format!(
            "Бюджет: ~{} свободно под GGUF (освобождено: +{:.2} ГБ)",
            format_gb(app.hw.recommended_model_budget_bytes),
            app.focus.freed_mb_total as f64 / 1024.0
        )),
        Line::from(format!(
            "Движок: {} | ctx={} | flash={} | kv_q8={}",
            app.cfg.engine.label(),
            app.cfg.context_size,
            app.cfg.flash_attn,
            app.cfg.kv_cache_q8
        )),
    ];
    let hw_panel = Paragraph::new(hw_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_DIM))
            .title(" Железо и профиль "),
    );
    frame.render_widget(hw_panel, right_rows[0]);

    let model_items: Vec<ListItem<'_>> = if app.models.is_empty() {
        vec![ListItem::new(format!(
            "Нет .gguf файлов в '{}' (положите модель или выберите CloudApi в [3])",
            app.cfg.models_dir
        ))]
    } else {
        app.models
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let marker = if i == app.selected_model_idx { "▶ " } else { "  " };
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("model.gguf");
                let size_gb = std::fs::metadata(p)
                    .map(|m| m.len() as f64 / (1024.0 * 1024.0 * 1024.0))
                    .unwrap_or(0.0);
                let style = if i == app.selected_model_idx {
                    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("{marker}{name} ({size_gb:.2} ГБ)")).style(style)
            })
            .collect()
    };

    let models_list = List::new(model_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_DIM))
            .title(" Локальные GGUF модели (↑/↓ выбор, R обновить) "),
    );
    frame.render_widget(models_list, right_rows[1]);

    render_footer(
        frame,
        chunks[2],
        "1/Enter: Чат  |  2: Focus Mode  |  3: Настройки  |  ↑/↓: Выбор модели  |  Ctrl+R: Восстановить окна  |  Q: Выход",
        &app.status_banner,
    );
}

fn render_focus_mode(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_top_bar(frame, chunks[0], app, "Режим «Только ИИ» (Focus Mode)");

    let total_ratio = if app.hw.ram_total_bytes > 0 {
        (app.hw.ram_used_bytes as f64 / app.hw.ram_total_bytes as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_DIM))
                .title(format!(
                    " Освобождено в сессии: +{:.2} ГБ RAM ",
                    app.focus.freed_mb_total as f64 / 1024.0
                )),
        )
        .gauge_style(Style::default().fg(CYAN).bg(Color::Rgb(20, 30, 44)))
        .ratio(total_ratio)
        .label(format!(
            "Занято {} / {} · Свободно под модель ~{}",
            format_gb(app.hw.ram_used_bytes),
            format_gb(app.hw.ram_total_bytes),
            format_gb(app.hw.recommended_model_budget_bytes),
        ));
    frame.render_widget(gauge, chunks[1]);

    let rows: Vec<Row<'_>> = app
        .focus
        .targets
        .iter()
        .map(|t| {
            let check = if t.keep { "[x] Оставить" } else { "[ ] Закрыть" };
            let status_color = match t.status {
                ProcessRowStatus::Kept => CYAN,
                ProcessRowStatus::Ready => Color::Yellow,
                ProcessRowStatus::Closing => BLUE,
                ProcessRowStatus::HungTimeout => Color::Red,
                ProcessRowStatus::Closed => Color::Green,
            };
            let title = format!("{} ({})", t.window_title, t.process_name);
            Row::new(vec![
                Cell::from(check),
                Cell::from(title),
                Cell::from(format!("{} МБ", t.ram_mb)),
                Cell::from(t.status.label()).style(Style::default().fg(status_color)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Min(30),
            Constraint::Length(12),
            Constraint::Length(18),
        ],
    )
    .header(
        Row::new(vec![
            "Оставить",
            "Заголовок окна / Процесс",
            "RAM (МБ)",
            "Статус",
        ])
        .style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_DIM))
            .title(" Активные окна и процессы (>80 МБ). Системные службы Windows защищены "),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(24, 48, 76))
            .add_modifier(Modifier::BOLD),
    );

    let mut table_state = TableState::default();
    if !app.focus.targets.is_empty() {
        table_state.select(Some(app.focus.selected_idx));
    }
    frame.render_stateful_widget(table, chunks[2], &mut table_state);

    render_footer(
        frame,
        chunks[3],
        "Space: [x] Оставить  |  A: Все/Снять  |  Enter: Очистить RAM  |  F: Kill зависших  |  Ctrl+R: Вернуть  |  Esc: Назад",
        &app.focus.last_status_msg,
    );
}

fn render_chat(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let ttft_str = app
        .speed_hud
        .ttft_ms
        .map(|v| format!("{v} мс"))
        .unwrap_or_else(|| "—".to_string());
    let hud_line = Line::from(vec![
        Span::styled(" HUD ", Style::default().fg(Color::Black).bg(BLUE).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(
                "  {:.1} tok/s  |  TTFT: {}  |  Токенов: {}  |  RAM: {}/{}  |  CPU: {:.0}%  |  {}",
                app.speed_hud.tok_per_sec,
                ttft_str,
                app.speed_hud.tokens_generated,
                format_gb(app.hw.ram_used_bytes),
                format_gb(app.hw.ram_total_bytes),
                app.hw.cpu_usage_percent,
                app.speed_hud.backend_label,
            ),
            Style::default().fg(CYAN),
        ),
    ]);
    let hud = Paragraph::new(hud_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_DIM)),
    );
    frame.render_widget(hud, chunks[0]);

    let mut chat_lines = Vec::new();
    if app.chat_history.is_empty() {
        chat_lines.push(Line::from(Span::styled(
            "Введите сообщение ниже и нажмите Enter для начала диалога.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for msg in &app.chat_history {
            let (prefix, color) = if msg.role == "user" {
                ("Вы > ", BLUE)
            } else {
                ("ИИ > ", CYAN)
            };
            chat_lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::raw(&msg.content),
            ]));
            chat_lines.push(Line::from(""));
        }
    }

    let chat_box = Paragraph::new(chat_lines)
        .wrap(Wrap { trim: false })
        .scroll((app.chat_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_DIM))
                .title(" Чат "),
        );
    frame.render_widget(chat_box, chunks[1]);

    let prompt_title = if app.speed_hud.is_streaming {
        " Генерация ответа... "
    } else {
        " Запрос (Enter — отправить, Esc — назад) "
    };
    let input_box = Paragraph::new(app.chat_input.as_str()).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(CYAN))
            .title(prompt_title),
    );
    frame.render_widget(input_box, chunks[2]);

    let footer = Paragraph::new(app.status_banner.as_str())
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_DIM)),
        );
    frame.render_widget(footer, chunks[3]);
}

fn render_settings(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_top_bar(frame, chunks[0], app, "Настройки движка и профили");

    let items: Vec<ListItem<'_>> = SettingField::ALL
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let (name, val) = match field {
                SettingField::Engine => ("Движок инференса", app.cfg.engine.label().to_string()),
                SettingField::ContextSize => ("Контекст (--ctx-size)", app.cfg.context_size.to_string()),
                SettingField::FlashAttn => (
                    "Flash Attention (--flash-attn)",
                    if app.cfg.flash_attn { "Вкл" } else { "Выкл" }.to_string(),
                ),
                SettingField::KvCacheQ8 => (
                    "KV-кэш q8_0",
                    if app.cfg.kv_cache_q8 { "Вкл" } else { "Выкл" }.to_string(),
                ),
                SettingField::Mlock => (
                    "Блокировка в RAM (--mlock)",
                    if app.cfg.mlock { "Вкл" } else { "Выкл" }.to_string(),
                ),
                SettingField::PowerPlan => (
                    "Авто High Performance (powercfg)",
                    if app.cfg.high_performance_power {
                        "Вкл"
                    } else {
                        "Выкл"
                    }
                    .to_string(),
                ),
                SettingField::CloudProvider => (
                    "Облачный провайдер (300+ tok/s)",
                    format!(
                        "{} ({})",
                        app.cfg.cloud_provider.label(),
                        app.cfg.cloud_model
                    ),
                ),
            };
            let prefix = if i == app.setting_idx { "▶ " } else { "  " };
            let style = if i == app.setting_idx {
                Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{prefix}{name:<38} : {val}")).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_DIM))
            .title(" Профиль запуска (Enter / Space — изменить, Esc — в меню) "),
    );
    frame.render_widget(list, chunks[1]);

    render_footer(
        frame,
        chunks[2],
        "↑/↓: Выбор параметра  |  Enter/Space: Переключить  |  Esc: Назад",
        &app.status_banner,
    );
}

fn render_exit_restore_modal(frame: &mut Frame<'_>, app: &App) {
    let area = centered_rect(64, 34, frame.area());
    frame.render_widget(Clear, area);

    let count = app.focus.last_snapshot.entries.len();
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Восстановить {count} закрытых приложений перед выходом?"),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Снапшот сохранён в %USERPROFILE%\\.llmlab\\session_snapshot.json"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [Y] Да, восстановить и выйти   ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            Span::styled("  [N] Выйти без восстановления   ", Style::default().fg(Color::Yellow)),
            Span::styled("  [Esc] Отмена", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let modal = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(CYAN))
                .title(" Выход из llmLab "),
        );
    frame.render_widget(modal, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
