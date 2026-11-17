use crate::config::{AppConfig, EngineKind};
use crate::hardware::CpuTopology;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};

#[cfg(windows)]
const HIGH_PERFORMANCE_GUID: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(windows)]
const HIGH_PRIORITY_CLASS_FLAG: u32 = 0x0000_0080;

/// Computes the optimal inference thread count:
/// On hybrid Intel 12/13/14+ Gen CPUs, uses only fast P-cores to avoid E-core barrier stalls.
/// On uniform CPUs, uses physical core count (memory-bandwidth optimal for LLMs).
pub fn optimal_threads(cpu: &CpuTopology) -> usize {
    cpu.p_cores.max(1)
}

/// RAII guard that switches Windows power plan to High Performance while a model runs,
/// and restores the user's previous power plan when dropped.
#[derive(Debug)]
pub struct PowerPlanGuard {
    previous_guid: Option<String>,
}

impl PowerPlanGuard {
    pub fn activate_if_enabled(enabled: bool) -> Self {
        if !enabled {
            return Self {
                previous_guid: None,
            };
        }

        #[cfg(windows)]
        {
            let prev = get_active_power_guid();
            if prev.as_deref() != Some(HIGH_PERFORMANCE_GUID) {
                let _ = set_power_guid(HIGH_PERFORMANCE_GUID);
            }
            Self {
                previous_guid: prev,
            }
        }

        #[cfg(not(windows))]
        {
            Self {
                previous_guid: None,
            }
        }
    }
}

impl Drop for PowerPlanGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Some(ref guid) = self.previous_guid {
            if guid != HIGH_PERFORMANCE_GUID {
                let _ = set_power_guid(guid);
            }
        }
    }
}

#[cfg(windows)]
fn get_active_power_guid() -> Option<String> {
    use std::os::windows::process::CommandExt;

    let out = std::process::Command::new("powercfg")
        .arg("/getactivescheme")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    extract_guid(&text)
}

#[cfg(windows)]
fn set_power_guid(guid: &str) -> bool {
    use std::os::windows::process::CommandExt;

    std::process::Command::new("powercfg")
        .args(["/setactive", guid])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .is_ok_and(|s| s.success())
}

pub fn extract_guid(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| c == '(' || c == ')' || c == ':');
        if cleaned.len() == 36 && cleaned.chars().filter(|&c| c == '-').count() == 4 {
            return Some(cleaned.to_ascii_lowercase());
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct EngineLaunchSpec {
    pub binary: String,
    pub args: Vec<String>,
}

pub fn build_engine_launch_spec(
    cfg: &AppConfig,
    model_path: &Path,
    cpu: &CpuTopology,
) -> EngineLaunchSpec {
    let threads = optimal_threads(cpu).to_string();
    let ctx = cfg.context_size.to_string();
    let port = cfg.server_port.to_string();
    let model_str = model_path.to_string_lossy().to_string();

    match cfg.engine {
        EngineKind::LlamaCppAvx2 | EngineKind::LlamaCppVulkan => {
            let mut args = vec![
                "-m".to_string(),
                model_str,
                "--host".to_string(),
                "127.0.0.1".to_string(),
                "--port".to_string(),
                port,
                "-t".to_string(),
                threads,
                "-c".to_string(),
                ctx,
            ];
            if cfg.flash_attn {
                args.push("--flash-attn".to_string());
            }
            if cfg.kv_cache_q8 {
                args.extend([
                    "--cache-type-k".to_string(),
                    "q8_0".to_string(),
                    "--cache-type-v".to_string(),
                    "q8_0".to_string(),
                ]);
            }
            if cfg.mlock {
                args.push("--mlock".to_string());
            }
            let ngl = if cfg.engine == EngineKind::LlamaCppVulkan {
                if cfg.gpu_layers > 0 {
                    cfg.gpu_layers
                } else {
                    99
                }
            } else {
                0
            };
            args.extend(["-ngl".to_string(), ngl.to_string()]);

            EngineLaunchSpec {
                binary: cfg.llama_server_path.clone(),
                args,
            }
        }
        EngineKind::OpenVino => EngineLaunchSpec {
            binary: cfg.openvino_cli_path.clone(),
            args: vec![
                "--model".to_string(),
                model_str,
                "--device".to_string(),
                "AUTO".to_string(),
                "--port".to_string(),
                port,
                "--max-context".to_string(),
                ctx,
            ],
        },
        EngineKind::BitNet => EngineLaunchSpec {
            binary: cfg.bitnet_cli_path.clone(),
            args: vec![
                "-m".to_string(),
                model_str,
                "--host".to_string(),
                "127.0.0.1".to_string(),
                "--port".to_string(),
                port,
                "-t".to_string(),
                threads,
                "-c".to_string(),
                ctx,
            ],
        },
        EngineKind::CloudApi => EngineLaunchSpec {
            binary: String::new(),
            args: Vec::new(),
        },
    }
}

pub fn spawn_tuned_engine(
    spec: &EngineLaunchSpec,
    high_priority: bool,
) -> Result<Child> {
    let mut cmd = Command::new(&spec.binary);
    cmd.args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    #[cfg(windows)]
    {
        let mut flags = CREATE_NO_WINDOW;
        if high_priority {
            flags |= HIGH_PRIORITY_CLASS_FLAG;
        }
        cmd.creation_flags(flags);
    }

    #[cfg(not(windows))]
    let _ = high_priority;

    cmd.spawn()
        .with_context(|| format!("Не удалось запустить движок '{}'. Проверьте путь в config.toml.", spec.binary))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_cpu_picks_only_p_cores() {
        let cpu = CpuTopology {
            brand: "Intel Core i5-12400H".to_string(),
            p_cores: 4,
            e_cores: 8,
            logical_threads: 16,
            avx2: true,
            neon: false,
        };
        assert_eq!(optimal_threads(&cpu), 4);
    }

    #[test]
    fn extracts_powercfg_guid() {
        let out = "Power Scheme GUID: 381b4222-f694-41f0-9685-ff5bb260df2e  (Balanced)";
        assert_eq!(
            extract_guid(out).as_deref(),
            Some("381b4222-f694-41f0-9685-ff5bb260df2e")
        );
    }

    #[test]
    fn builds_llama_server_flags_correctly() {
        let cfg = AppConfig {
            engine: EngineKind::LlamaCppAvx2,
            context_size: 4096,
            flash_attn: true,
            kv_cache_q8: true,
            mlock: true,
            ..AppConfig::default()
        };
        let cpu = CpuTopology {
            brand: "Ryzen 5 5500U".to_string(),
            p_cores: 6,
            e_cores: 0,
            logical_threads: 12,
            avx2: true,
            neon: false,
        };
        let spec = build_engine_launch_spec(&cfg, Path::new("models/qwen2.5-3b.gguf"), &cpu);
        assert!(spec.args.contains(&"--flash-attn".to_string()));
        assert!(spec.args.contains(&"q8_0".to_string()));
        assert!(spec.args.contains(&"--mlock".to_string()));
        assert!(spec.args.contains(&"6".to_string()));
    }
}
