use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    LlamaCppAvx2,
    LlamaCppVulkan,
    OpenVino,
    BitNet,
    CloudApi,
}

impl EngineKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::LlamaCppAvx2 => "llama.cpp (AVX2 CPU)",
            Self::LlamaCppVulkan => "llama.cpp (Vulkan iGPU/GPU)",
            Self::OpenVino => "OpenVINO GenAI (Intel)",
            Self::BitNet => "bitnet.cpp (1.58-bit CPU)",
            Self::CloudApi => "Cloud Boost (300+ tok/s)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::LlamaCppAvx2 => Self::LlamaCppVulkan,
            Self::LlamaCppVulkan => Self::OpenVino,
            Self::OpenVino => Self::BitNet,
            Self::BitNet => Self::CloudApi,
            Self::CloudApi => Self::LlamaCppAvx2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudProvider {
    Groq,
    Cerebras,
    OpenRouter,
    Custom,
}

impl CloudProvider {
    pub fn label(self) -> &'static str {
        match self {
            Self::Groq => "Groq (LPU)",
            Self::Cerebras => "Cerebras (Wafer-Scale)",
            Self::OpenRouter => "OpenRouter",
            Self::Custom => "Custom OpenAI-Compatible",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            Self::Groq => "https://api.groq.com/openai/v1",
            Self::Cerebras => "https://api.cerebras.ai/v1",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::Custom => "http://127.0.0.1:8080/v1",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::Groq => "llama-3.3-70b-versatile",
            Self::Cerebras => "llama3.1-8b",
            Self::OpenRouter => "meta-llama/llama-3.3-70b-instruct",
            Self::Custom => "local-model",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Groq => Self::Cerebras,
            Self::Cerebras => Self::OpenRouter,
            Self::OpenRouter => Self::Custom,
            Self::Custom => Self::Groq,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub models_dir: String,
    pub engine: EngineKind,
    pub llama_server_path: String,
    pub llama_cli_path: String,
    pub openvino_cli_path: String,
    pub bitnet_cli_path: String,
    pub server_port: u16,
    pub context_size: u32,
    pub flash_attn: bool,
    pub kv_cache_q8: bool,
    pub mlock: bool,
    pub gpu_layers: i32,
    pub high_performance_power: bool,
    pub high_process_priority: bool,
    pub cloud_provider: CloudProvider,
    pub cloud_base_url: String,
    pub cloud_model: String,
    pub cloud_api_key_env: String,
    pub system_prompt: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let bin = |base: &str| -> String {
            if cfg!(windows) {
                format!("{base}.exe")
            } else {
                base.to_string()
            }
        };
        Self {
            models_dir: "./models".to_string(),
            engine: EngineKind::LlamaCppAvx2,
            llama_server_path: bin("llama-server"),
            llama_cli_path: bin("llama-cli"),
            openvino_cli_path: bin("openvino-genai-cli"),
            bitnet_cli_path: bin("bitnet-server"),
            server_port: 8089,
            context_size: 4096,
            flash_attn: true,
            kv_cache_q8: true,
            mlock: false,
            gpu_layers: 0,
            high_performance_power: true,
            high_process_priority: true,
            cloud_provider: CloudProvider::Groq,
            cloud_base_url: CloudProvider::Groq.default_base_url().to_string(),
            cloud_model: CloudProvider::Groq.default_model().to_string(),
            cloud_api_key_env: "LLMLAB_API_KEY".to_string(),
            system_prompt: "You are a fast, concise, and technically accurate local AI assistant."
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeepAppsStore {
    pub keep_processes: Vec<String>,
}

impl KeepAppsStore {
    pub fn as_lowercase_set(&self) -> HashSet<String> {
        self.keep_processes
            .iter()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClosedAppEntry {
    pub process_name: String,
    pub window_title: String,
    pub exe_path: String,
    pub args: Vec<String>,
    pub freed_mb: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSnapshot {
    pub created_at: String,
    pub entries: Vec<ClosedAppEntry>,
}

pub fn llmlab_data_dir() -> PathBuf {
    let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(".llmlab")
}

pub fn keep_apps_path() -> PathBuf {
    llmlab_data_dir().join("keep_apps.json")
}

pub fn session_snapshot_path() -> PathBuf {
    llmlab_data_dir().join("session_snapshot.json")
}

pub fn app_install_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
}

fn is_protected_system_dir(dir: &Path) -> bool {
    let low = dir.to_string_lossy().to_ascii_lowercase();
    if cfg!(windows) {
        low.ends_with(r"\windows\system32")
            || low.ends_with(r"\windows\syswow64")
            || low == r"c:\windows"
    } else {
        matches!(low.as_str(), "/" | "/bin" | "/usr" | "/usr/bin" | "/system" | "/sbin")
    }
}

fn dir_is_writable(dir: &Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(format!(".llmlab_write_probe_{}", std::process::id()));
    match fs::OpenOptions::new().write(true).create_new(true).open(&probe) {
        Ok(file) => {
            drop(file);
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Resolves relative paths like `./models` or `./workspace` to a guaranteed-writable
/// location: existing local dir (if writable & non-System32), next to `llmLab.exe`,
/// or `%USERPROFILE%\.llmlab\<subpath>` fallback.
pub fn resolve_app_subdir(configured: &str) -> PathBuf {
    let trimmed = configured.trim();
    if trimmed.is_empty() {
        return resolve_app_subdir("models");
    }

    let raw_path = Path::new(trimmed);
    if raw_path.is_absolute() {
        return raw_path.to_path_buf();
    }

    let rel_clean = trimmed
        .trim_start_matches("./")
        .trim_start_matches(r".\");
    let rel = if rel_clean.is_empty() { "models" } else { rel_clean };

    let safe_cwd = std::env::current_dir()
        .ok()
        .filter(|d| !is_protected_system_dir(d));

    if let Some(ref cwd) = safe_cwd {
        let local_candidate = cwd.join(rel);
        if local_candidate.is_dir() && dir_is_writable(&local_candidate) {
            return local_candidate;
        }
    }

    if let Some(exe_dir) = app_install_dir().filter(|d| !is_protected_system_dir(d)) {
        let beside_exe = exe_dir.join(rel);
        if dir_is_writable(&beside_exe) {
            return beside_exe;
        }
    }

    if let Some(cwd) = safe_cwd {
        let local_candidate = cwd.join(rel);
        if dir_is_writable(&local_candidate) {
            return local_candidate;
        }
    }

    let fallback = llmlab_data_dir().join(rel);
    let _ = fs::create_dir_all(&fallback);
    fallback
}

pub fn config_file_path() -> PathBuf {
    let cwd_safe = std::env::current_dir()
        .ok()
        .is_some_and(|d| !is_protected_system_dir(&d));
    let local = PathBuf::from("config.toml");
    if cwd_safe && local.exists() {
        return local;
    }
    if let Some(exe_cfg) = app_install_dir().map(|d| d.join("config.toml")) {
        if exe_cfg.exists() {
            return exe_cfg;
        }
    }
    llmlab_data_dir().join("config.toml")
}

pub fn load_config(path: Option<&Path>) -> AppConfig {
    let target = path
        .map(PathBuf::from)
        .unwrap_or_else(config_file_path);

    if let Ok(raw) = fs::read_to_string(&target) {
        if let Ok(parsed) = toml::from_str::<AppConfig>(&raw) {
            return parsed;
        }
    }
    AppConfig::default()
}

pub fn save_config(cfg: &AppConfig, path: Option<&Path>) -> Result<()> {
    let target = path
        .map(PathBuf::from)
        .unwrap_or_else(|| llmlab_data_dir().join("config.toml"));
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }
    let serialized = toml::to_string_pretty(cfg).context("Failed to serialize config")?;
    fs::write(&target, serialized).context("Failed to write config.toml")?;
    Ok(())
}

pub fn load_keep_apps(path: Option<&Path>) -> KeepAppsStore {
    let target = path.map(PathBuf::from).unwrap_or_else(keep_apps_path);
    if let Ok(raw) = fs::read_to_string(&target) {
        if let Ok(store) = serde_json::from_str::<KeepAppsStore>(&raw) {
            return store;
        }
    }
    KeepAppsStore {
        keep_processes: vec![
            "discord.exe".to_string(),
            "telegram.exe".to_string(),
            "code.exe".to_string(),
        ],
    }
}

pub fn save_keep_apps(store: &KeepAppsStore, path: Option<&Path>) -> Result<()> {
    let target = path.map(PathBuf::from).unwrap_or_else(keep_apps_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).context("Failed to create ~/.llmlab directory")?;
    }
    let json = serde_json::to_string_pretty(store).context("Failed to serialize keep_apps")?;
    fs::write(&target, json).context("Failed to write keep_apps.json")?;
    Ok(())
}

pub fn load_session_snapshot(path: Option<&Path>) -> SessionSnapshot {
    let target = path
        .map(PathBuf::from)
        .unwrap_or_else(session_snapshot_path);
    if let Ok(raw) = fs::read_to_string(&target) {
        if let Ok(snapshot) = serde_json::from_str::<SessionSnapshot>(&raw) {
            return snapshot;
        }
    }
    SessionSnapshot::default()
}

pub fn save_session_snapshot(snapshot: &SessionSnapshot, path: Option<&Path>) -> Result<()> {
    let target = path
        .map(PathBuf::from)
        .unwrap_or_else(session_snapshot_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).context("Failed to create ~/.llmlab directory")?;
    }
    let json =
        serde_json::to_string_pretty(snapshot).context("Failed to serialize session snapshot")?;
    fs::write(&target, json).context("Failed to write session_snapshot.json")?;
    Ok(())
}

/// Scans resolved `models_dir` for local `.gguf` files sorted by filename.
pub fn discover_gguf_models(models_dir: &str) -> Vec<PathBuf> {
    let dir = resolve_app_subdir(models_dir);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut models: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"))
        })
        .collect();
    models.sort();
    models
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("llmlab_test_{nanos}_{name}"))
    }

    #[test]
    fn config_round_trip_toml() {
        let path = unique_temp_file("config.toml");
        let cfg = AppConfig {
            context_size: 8192,
            kv_cache_q8: false,
            engine: EngineKind::LlamaCppVulkan,
            ..AppConfig::default()
        };
        save_config(&cfg, Some(&path)).expect("save config");
        let loaded = load_config(Some(&path));
        assert_eq!(loaded.context_size, 8192);
        assert!(!loaded.kv_cache_q8);
        assert_eq!(loaded.engine, EngineKind::LlamaCppVulkan);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn keep_apps_and_snapshot_round_trip_json() {
        let keep_path = unique_temp_file("keep.json");
        let snap_path = unique_temp_file("snap.json");

        let keep = KeepAppsStore {
            keep_processes: vec!["Code.exe".to_string(), "Telegram.exe".to_string()],
        };
        save_keep_apps(&keep, Some(&keep_path)).expect("save keep_apps");
        let loaded_keep = load_keep_apps(Some(&keep_path));
        let set = loaded_keep.as_lowercase_set();
        assert!(set.contains("code.exe"));
        assert!(set.contains("telegram.exe"));

        let snap = SessionSnapshot {
            created_at: "2026-11-17T00:00:00Z".to_string(),
            entries: vec![ClosedAppEntry {
                process_name: "chrome.exe".to_string(),
                window_title: "Google Chrome".to_string(),
                exe_path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string(),
                args: vec!["--restore-last-session".to_string()],
                freed_mb: 1450,
            }],
        };
        save_session_snapshot(&snap, Some(&snap_path)).expect("save snapshot");
        let loaded_snap = load_session_snapshot(Some(&snap_path));
        assert_eq!(loaded_snap, snap);

        let _ = fs::remove_file(keep_path);
        let _ = fs::remove_file(snap_path);
    }

    #[test]
    fn resolves_relative_models_dir_cleanly() {
        let resolved = resolve_app_subdir("./models");
        assert!(resolved.ends_with("models"));
        let abs = std::env::temp_dir().join("llmlab_abs_models");
        assert_eq!(resolve_app_subdir(&abs.to_string_lossy()), abs);
    }
}
