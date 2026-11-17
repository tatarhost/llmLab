use crate::agent::{AgentRole, SkillFlags};
use crate::chat::{spawn_chat_stream, ChatMessage, SpeedHud, StreamEvent};
use crate::config::{
    discover_gguf_models, load_config, resolve_app_subdir, save_config, AppConfig, EngineKind,
};
use crate::focus_mode::FocusModeState;
use crate::hardware::HardwareProfile;
use crate::hf::{
    auto_pick_curated_models, spawn_hf_download, spawn_hf_list_files, spawn_hf_search, HfEvent,
    HfFileEntry, HfModelCandidate,
};
use crate::tuner::{build_engine_launch_spec, spawn_tuned_engine, PowerPlanGuard};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use sysinfo::System;
use tokio::process::Child;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Splash,
    Dashboard,
    Models,
    Chat,
    Skills,
    FocusMode,
    Settings,
}

impl Screen {
    pub const NAV_TABS: [Self; 6] = [
        Self::Dashboard,
        Self::Models,
        Self::Chat,
        Self::Skills,
        Self::FocusMode,
        Self::Settings,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Splash => "Запуск",
            Self::Dashboard => "Обзор",
            Self::Models => "Модели HF",
            Self::Chat => "Агент / Чат",
            Self::Skills => "Скиллы",
            Self::FocusMode => "Только ИИ",
            Self::Settings => "Настройки",
        }
    }

    pub fn next_tab(self) -> Self {
        let Some(pos) = Self::NAV_TABS.iter().position(|&s| s == self) else {
            return Self::Dashboard;
        };
        Self::NAV_TABS[(pos + 1) % Self::NAV_TABS.len()]
    }

    pub fn prev_tab(self) -> Self {
        let Some(pos) = Self::NAV_TABS.iter().position(|&s| s == self) else {
            return Self::Dashboard;
        };
        Self::NAV_TABS[(pos + Self::NAV_TABS.len() - 1) % Self::NAV_TABS.len()]
    }
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

#[derive(Debug, Clone, Default)]
pub struct DownloadProgressState {
    pub active: bool,
    pub file_name: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed_mb_s: f64,
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
    // Hugging Face Hub state
    pub hf_tx: mpsc::UnboundedSender<HfEvent>,
    pub hf_query: String,
    pub hf_search_focused: bool,
    pub hf_candidates: Vec<HfModelCandidate>,
    pub hf_selected_idx: usize,
    pub hf_repo_files: Vec<HfFileEntry>,
    pub hf_active_repo: Option<String>,
    pub hf_file_idx: usize,
    pub hf_in_file_picker: bool,
    pub hf_download: DownloadProgressState,
    // Agent & Skills state
    pub agent_role: AgentRole,
    pub skills: SkillFlags,
    pub skill_cursor: usize,
    pub agent_logs: Vec<String>,
    pub models_dir_resolved: PathBuf,
    pub workspace_dir: PathBuf,
}

impl App {
    pub fn new(
        skip_splash: bool,
        start_in_focus: bool,
        use_utf8: bool,
        stream_tx: mpsc::UnboundedSender<StreamEvent>,
        hf_tx: mpsc::UnboundedSender<HfEvent>,
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

        let hf_candidates = auto_pick_curated_models(&hw);

        let models_dir_resolved = resolve_app_subdir(&cfg.models_dir);
        let workspace_dir = resolve_app_subdir("./workspace");

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
            status_banner: "Стрелки ←/→ переключают вкладки · ↑/↓ выбор · Enter запуск".to_string(),
            show_exit_restore_modal: false,
            should_quit: false,
            engine_child: None,
            power_guard: None,
            stream_tx,
            hf_tx,
            hf_query: String::new(),
            hf_search_focused: false,
            hf_candidates,
            hf_selected_idx: 0,
            hf_repo_files: Vec::new(),
            hf_active_repo: None,
            hf_file_idx: 0,
            hf_in_file_picker: false,
            hf_download: DownloadProgressState::default(),
            agent_role: AgentRole::AppBuilder,
            skills: SkillFlags::default(),
            skill_cursor: 0,
            agent_logs: vec![format!(
                "Агент готов: веб-поиск и песочница {} активны.",
                workspace_dir.display()
            )],
            models_dir_resolved,
            workspace_dir,
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

    pub fn delete_selected_local_model(&mut self) {
        let Some(path) = self.selected_model().cloned() else {
            self.status_banner = "Нет выбранной локальной модели для удаления.".to_string();
            return;
        };
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("model.gguf")
            .to_string();
        match fs::remove_file(&path) {
            Ok(_) => {
                self.refresh_models();
                self.status_banner = format!("Удалена локальная модель: {name}");
            }
            Err(err) => {
                self.status_banner = format!("Не удалось удалить {name}: {err}");
            }
        }
    }

    pub fn trigger_hf_search(&mut self) {
        self.hf_in_file_picker = false;
        self.hf_repo_files.clear();
        self.status_banner = if self.hf_query.trim().is_empty() {
            "Поиск популярных GGUF моделей на Hugging Face...".to_string()
        } else {
            format!("Поиск на Hugging Face: '{}'...", self.hf_query.trim())
        };
        spawn_hf_search(
            self.hf_query.clone(),
            self.hw.recommended_model_budget_bytes,
            self.hw.gpu.vulkan_available,
            self.hf_tx.clone(),
        );
    }

    pub fn trigger_hw_auto_pick(&mut self) {
        self.hf_in_file_picker = false;
        self.hf_repo_files.clear();
        self.hf_candidates = auto_pick_curated_models(&self.hw);
        self.hf_selected_idx = 0;
        self.status_banner = "Выполнен автоподбор моделей под RAM/CPU/GPU этого ПК.".to_string();
    }

    pub fn open_or_download_hf_item(&mut self) {
        if self.hf_in_file_picker {
            if let (Some(repo), Some(file)) = (
                self.hf_active_repo.clone(),
                self.hf_repo_files.get(self.hf_file_idx).cloned(),
            ) {
                self.hf_download = DownloadProgressState {
                    active: true,
                    file_name: file.path.clone(),
                    downloaded: 0,
                    total: file.size_bytes,
                    speed_mb_s: 0.0,
                };
                self.status_banner = format!("Загрузка {} из {}...", file.path, repo);
                spawn_hf_download(
                    repo,
                    file.path,
                    self.cfg.models_dir.clone(),
                    self.hf_tx.clone(),
                );
            }
            return;
        }

        let Some(candidate) = self.hf_candidates.get(self.hf_selected_idx).cloned() else {
            return;
        };

        if let Some(rec_file) = candidate.recommended_file {
            self.hf_download = DownloadProgressState {
                active: true,
                file_name: rec_file.clone(),
                downloaded: 0,
                total: 0,
                speed_mb_s: 0.0,
            };
            self.status_banner = format!("Прямое скачивание {rec_file} с Hugging Face...");
            spawn_hf_download(
                candidate.repo_id,
                rec_file,
                self.cfg.models_dir.clone(),
                self.hf_tx.clone(),
            );
        } else {
            self.status_banner = format!("Получение GGUF-файлов из {}...", candidate.repo_id);
            spawn_hf_list_files(candidate.repo_id, self.hf_tx.clone());
        }
    }

    pub fn inspect_hf_repo_files(&mut self) {
        if let Some(candidate) = self.hf_candidates.get(self.hf_selected_idx).cloned() {
            self.status_banner = format!("Запрос квантований {}...", candidate.repo_id);
            spawn_hf_list_files(candidate.repo_id, self.hf_tx.clone());
        }
    }

    pub fn handle_hf_event(&mut self, event: HfEvent) {
        match event {
            HfEvent::SearchResults { query, items } => {
                self.hf_candidates = items;
                self.hf_selected_idx = 0;
                self.hf_in_file_picker = false;
                self.status_banner = format!(
                    "Hugging Face: найдено {} моделей{}",
                    self.hf_candidates.len(),
                    if query.trim().is_empty() {
                        String::new()
                    } else {
                        format!(" по '{query}'")
                    }
                );
            }
            HfEvent::RepoFiles { repo_id, files } => {
                self.hf_active_repo = Some(repo_id.clone());
                self.hf_repo_files = files;
                self.hf_file_idx = 0;
                self.hf_in_file_picker = true;
                self.status_banner = format!(
                    "{}: найдено {} .gguf файлов (↑/↓ выбор, Enter скачать, Esc назад)",
                    repo_id,
                    self.hf_repo_files.len()
                );
            }
            HfEvent::DownloadProgress {
                file_name,
                downloaded,
                total,
                speed_mb_s,
            } => {
                self.hf_download = DownloadProgressState {
                    active: true,
                    file_name,
                    downloaded,
                    total,
                    speed_mb_s,
                };
            }
            HfEvent::DownloadDone { file_name, saved_path } => {
                self.hf_download.active = false;
                self.refresh_models();
                if let Some(pos) = self.models.iter().position(|p| p == &saved_path) {
                    self.selected_model_idx = pos;
                }
                self.status_banner = format!(
                    "Модель скачана в {}: {file_name} (готова к запуску)",
                    saved_path.display()
                );
            }
            HfEvent::Error(err) => {
                self.hf_download.active = false;
                self.status_banner = err;
            }
        }
    }

    pub fn toggle_skill_at_cursor(&mut self) {
        match self.skill_cursor {
            0 => self.agent_role = self.agent_role.next(),
            1 => self.skills.web_search = !self.skills.web_search,
            2 => self.skills.file_builder = !self.skills.file_builder,
            3 => self.skills.shell_exec = !self.skills.shell_exec,
            4 => self.skills.auto_approve_tools = !self.skills.auto_approve_tools,
            _ => {}
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
                "В '{}' нет .gguf. Скачайте модель во вкладке [Модели HF] (→) или выберите CloudApi.",
                self.models_dir_resolved.display()
            );
            return;
        };

        self.stop_engine();

        // Auto-detect best engine based on user hardware (OpenVINO for Intel, Vulkan for GPU, AVX2 for CPU)
        self.cfg.engine = crate::tuner::auto_select_optimal_engine(&self.hw);

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
        spawn_chat_stream(
            self.cfg.clone(),
            history_to_send,
            self.agent_role,
            self.skills.clone(),
            self.workspace_dir.clone(),
            self.stream_tx.clone(),
        );
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
            StreamEvent::ToolLog(entry) => {
                self.agent_logs.push(entry);
                if self.agent_logs.len() > 24 {
                    let excess = self.agent_logs.len() - 24;
                    self.agent_logs.drain(..excess);
                }
            }
            StreamEvent::AppendAssistantMessage => {
                self.chat_history.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: String::new(),
                });
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
