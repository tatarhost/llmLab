use crate::chat::{spawn_chat_stream, ChatMessage, SpeedHud, StreamEvent};
use crate::config::{
    discover_gguf_models, load_config, save_config, AppConfig, EngineKind,
};
use crate::focus_mode::FocusModeState;
use crate::hardware::HardwareProfile;
use crate::tuner::{build_engine_launch_spec, spawn_tuned_engine, PowerPlanGuard};
use std::path::PathBuf;
use std::time::Instant;
use sysinfo::System;
use tokio::process::Child;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Splash,
    Dashboard,
    FocusMode,
    Chat,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingField {
    Engine,
    ContextSize,
    FlashAttn,
    KvCacheQ8,
    Mlock,
    PowerPlan,
    CloudProvider,
}

impl SettingField {
    pub const ALL: [Self; 7] = [
        Self::Engine,
        Self::ContextSize,
        Self::FlashAttn,
        Self::KvCacheQ8,
        Self::Mlock,
        Self::PowerPlan,
        Self::CloudProvider,
    ];
}

pub struct App {
    pub screen: Screen,
    pub use_utf8: bool,
    pub splash_started: Instant,
    pub hw: HardwareProfile,
    pub sys: System,
    pub cfg: AppConfig,
    pub models: Vec<PathBuf>,
    pub selected_model_idx: usize,
    pub focus: FocusModeState,
    pub setting_idx: usize,
    pub chat_history: Vec<ChatMessage>,
    pub chat_input: String,
    pub chat_scroll: u16,
    pub speed_hud: SpeedHud,
    pub status_banner: String,
    pub show_exit_restore_modal: bool,
    pub should_quit: bool,
    pub engine_child: Option<Child>,
    pub power_guard: Option<PowerPlanGuard>,
    pub stream_tx: mpsc::UnboundedSender<StreamEvent>,
}

impl App {
    pub fn new(
        skip_splash: bool,
        start_in_focus: bool,
        use_utf8: bool,
        stream_tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> Self {
        let cfg = load_config(None);
        let models = discover_gguf_models(&cfg.models_dir);
        let hw = HardwareProfile::detect();
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        let mut focus = FocusModeState::default();
        focus.refresh();

        let initial_screen = if skip_splash {
            if start_in_focus {
                Screen::FocusMode
            } else {
                Screen::Dashboard
            }
        } else {
            Screen::Splash
        };

        Self {
            screen: initial_screen,
            use_utf8,
            splash_started: Instant::now(),
            hw,
            sys,
            cfg,
            models,
            selected_model_idx: 0,
            focus,
            setting_idx: 0,
            chat_history: Vec::new(),
            chat_input: String::new(),
            chat_scroll: 0,
            speed_hud: SpeedHud::default(),
            status_banner: "Готово к работе. Выберите модель GGUF или откройте Focus Mode.".to_string(),
            show_exit_restore_modal: false,
            should_quit: false,
            engine_child: None,
            power_guard: None,
            stream_tx,
        }
    }

    pub fn selected_model(&self) -> Option<&PathBuf> {
        self.models.get(self.selected_model_idx)
    }

    pub fn refresh_models(&mut self) {
        self.models = discover_gguf_models(&self.cfg.models_dir);
        if self.selected_model_idx >= self.models.len() {
            self.selected_model_idx = self.models.len().saturating_sub(1);
        }
    }

    pub fn refresh_telemetry(&mut self) {
        self.hw.refresh_live(&mut self.sys);
    }

    pub fn start_engine_and_open_chat(&mut self) {
        if self.cfg.engine == EngineKind::CloudApi {
            self.speed_hud.backend_label = format!(
                "{} · {}",
                self.cfg.cloud_provider.label(),
                self.cfg.cloud_model
            );
            self.status_banner = format!("Облачный режим активен: {}", self.cfg.cloud_model);
            self.screen = Screen::Chat;
            return;
        }

        let Some(model_path) = self.selected_model().cloned() else {
            self.status_banner = format!(
                "В папке '{}' не найдено .gguf моделей. Поместите модель или выберите CloudApi в Настройках [3].",
                self.cfg.models_dir
            );
            return;
        };

        self.stop_engine();
        self.power_guard = Some(PowerPlanGuard::activate_if_enabled(
            self.cfg.high_performance_power,
        ));

        let spec = build_engine_launch_spec(&self.cfg, &model_path, &self.hw.cpu);
        match spawn_tuned_engine(&spec, self.cfg.high_process_priority) {
            Ok(child) => {
                let model_name = model_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("model.gguf")
                    .to_string();
                self.engine_child = Some(child);
                self.speed_hud.backend_label =
                    format!("{} · {}", self.cfg.engine.label(), model_name);
                self.status_banner = format!(
                    "Движок запущен ({} потоков, High Priority).",
                    crate::tuner::optimal_threads(&self.hw.cpu)
                );
                self.screen = Screen::Chat;
            }
            Err(err) => {
                self.power_guard = None;
                self.status_banner = format!("Ошибка запуска: {err}");
            }
        }
    }

    pub fn stop_engine(&mut self) {
        if let Some(mut child) = self.engine_child.take() {
            let _ = child.start_kill();
        }
        self.power_guard = None;
    }

    pub fn send_chat_prompt(&mut self) {
        let prompt = self.chat_input.trim().to_string();
        if prompt.is_empty() || self.speed_hud.is_streaming {
            return;
        }
        self.chat_input.clear();
        self.chat_history.push(ChatMessage {
            role: "user".to_string(),
            content: prompt,
        });
        self.chat_history.push(ChatMessage {
            role: "assistant".to_string(),
            content: String::new(),
        });
        self.speed_hud.is_streaming = true;
        self.speed_hud.tokens_generated = 0;
        self.speed_hud.tok_per_sec = 0.0;
        self.speed_hud.ttft_ms = None;

        let history_to_send = self.chat_history[..self.chat_history.len() - 1].to_vec();
        spawn_chat_stream(self.cfg.clone(), history_to_send, self.stream_tx.clone());
    }

    pub fn handle_stream_event(&mut self, event: StreamEvent) {
        match event {
            StreamEvent::Delta(delta) => {
                if let Some(last) = self.chat_history.last_mut() {
                    if last.role == "assistant" {
                        last.content.push_str(&delta);
                    }
                }
            }
            StreamEvent::Stats {
                tok_per_sec,
                ttft_ms,
                tokens_generated,
            } => {
                self.speed_hud.tok_per_sec = tok_per_sec;
                self.speed_hud.ttft_ms = ttft_ms;
                self.speed_hud.tokens_generated = tokens_generated;
            }
            StreamEvent::Done => {
                self.speed_hud.is_streaming = false;
            }
            StreamEvent::Error(err) => {
                self.speed_hud.is_streaming = false;
                if let Some(last) = self.chat_history.last_mut() {
                    if last.role == "assistant" && last.content.is_empty() {
                        last.content = format!("[Ошибка] {err}");
                    }
                }
                self.status_banner = err;
            }
        }
    }

    pub fn cycle_selected_setting(&mut self) {
        let field = SettingField::ALL[self.setting_idx];
        match field {
            SettingField::Engine => {
                self.cfg.engine = self.cfg.engine.next();
            }
            SettingField::ContextSize => {
                self.cfg.context_size = match self.cfg.context_size {
                    2048 => 4096,
                    4096 => 8192,
                    _ => 2048,
                };
            }
            SettingField::FlashAttn => self.cfg.flash_attn = !self.cfg.flash_attn,
            SettingField::KvCacheQ8 => self.cfg.kv_cache_q8 = !self.cfg.kv_cache_q8,
            SettingField::Mlock => self.cfg.mlock = !self.cfg.mlock,
            SettingField::PowerPlan => {
                self.cfg.high_performance_power = !self.cfg.high_performance_power;
            }
            SettingField::CloudProvider => {
                self.cfg.cloud_provider = self.cfg.cloud_provider.next();
                self.cfg.cloud_base_url = self.cfg.cloud_provider.default_base_url().to_string();
                self.cfg.cloud_model = self.cfg.cloud_provider.default_model().to_string();
            }
        }
        let _ = save_config(&self.cfg, None);
        self.status_banner = "Настройки сохранены в config.toml".to_string();
    }

    pub fn request_quit(&mut self) {
        if self.focus.freed_mb_total > 0 && !self.focus.last_snapshot.entries.is_empty() {
            self.show_exit_restore_modal = true;
        } else {
            self.stop_engine();
            self.should_quit = true;
        }
    }
}
