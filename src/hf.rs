use crate::hardware::HardwareProfile;
use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct HfModelCandidate {
    pub repo_id: String,
    pub recommended_file: Option<String>,
    pub est_size_gb: f64,
    pub downloads: u64,
    pub fit_label: &'static str,
    pub fits_pc: bool,
    pub est_tok_s: &'static str,
}

#[derive(Debug, Clone)]
pub struct HfFileEntry {
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub enum HfEvent {
    SearchResults {
        query: String,
        items: Vec<HfModelCandidate>,
    },
    RepoFiles {
        repo_id: String,
        files: Vec<HfFileEntry>,
    },
    DownloadProgress {
        file_name: String,
        downloaded: u64,
        total: u64,
        speed_mb_s: f64,
    },
    DownloadDone {
        file_name: String,
        saved_path: PathBuf,
    },
    Error(String),
}

#[derive(Debug, Deserialize)]
struct HfApiModel {
    id: String,
    #[serde(default)]
    downloads: u64,
}

#[derive(Debug, Deserialize)]
struct HfTreeNode {
    #[serde(rename = "type")]
    node_type: String,
    path: String,
    #[serde(default)]
    size: u64,
}

pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for b in input.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            b' ' => out.push_str("%20"),
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}

pub fn classify_fit(est_size_gb: f64, budget_gb: f64, has_vulkan: bool) -> (&'static str, bool, &'static str) {
    if est_size_gb <= 0.1 {
        return ("[Подбор]", true, "—");
    }
    if est_size_gb <= budget_gb * 0.62 {
        let speed = if has_vulkan { "18–35 tok/s" } else { "12–24 tok/s" };
        ("[OK · Быстро]", true, speed)
    } else if est_size_gb <= budget_gb * 0.92 {
        let speed = if has_vulkan { "8–16 tok/s" } else { "5–11 tok/s" };
        ("[OK · Норма]", true, speed)
    } else {
        ("[Мало RAM]", false, "< 3 tok/s")
    }
}

/// Estimates parameter count in billions from common Hugging Face GGUF repository names.
pub fn estimate_params_from_name(repo_id: &str) -> f64 {
    let lower = repo_id.to_ascii_lowercase();
    for (needle, size_gb) in [
        ("0.5b", 0.45),
        ("1b", 0.8),
        ("1.5b", 1.15),
        ("2b", 1.5),
        ("3b", 2.1),
        ("4b", 2.7),
        ("7b", 4.6),
        ("8b", 4.9),
        ("9b", 5.6),
        ("12b", 7.4),
        ("14b", 8.8),
        ("22b", 13.5),
        ("32b", 19.5),
        ("70b", 40.0),
    ] {
        if lower.contains(needle) {
            return size_gb;
        }
    }
    2.5
}

/// Instant local hardware-aware catalog so weak laptops immediately get optimal models.
pub fn auto_pick_curated_models(hw: &HardwareProfile) -> Vec<HfModelCandidate> {
    let budget_gb = (hw.recommended_model_budget_bytes as f64 / (1024.0 * 1024.0 * 1024.0)).max(1.2);
    let has_vulkan = hw.gpu.vulkan_available;

    let curated = [
        (
            "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
            "qwen2.5-1.5b-instruct-q4_k_m.gguf",
            1.12,
            245_000,
        ),
        (
            "Qwen/Qwen2.5-3B-Instruct-GGUF",
            "qwen2.5-3b-instruct-q4_k_m.gguf",
            2.10,
            410_000,
        ),
        (
            "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
            "qwen2.5-coder-3b-instruct-q4_k_m.gguf",
            2.10,
            180_000,
        ),
        (
            "bartowski/Llama-3.2-3B-Instruct-GGUF",
            "Llama-3.2-3B-Instruct-Q4_K_M.gguf",
            2.02,
            390_000,
        ),
        (
            "Qwen/Qwen2.5-7B-Instruct-GGUF",
            "qwen2.5-7b-instruct-q4_k_m.gguf",
            4.68,
            520_000,
        ),
        (
            "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
            "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
            4.68,
            260_000,
        ),
        (
            "bartowski/Meta-Llama-3.1-8B-Instruct-GGUF",
            "Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
            4.92,
            640_000,
        ),
        (
            "bartowski/Qwen2.5-14B-Instruct-GGUF",
            "Qwen2.5-14B-Instruct-Q4_K_M.gguf",
            8.99,
            195_000,
        ),
    ];

    let mut items: Vec<HfModelCandidate> = curated
        .into_iter()
        .map(|(repo, file, gb, downloads)| {
            let (fit_label, fits_pc, est_tok_s) = classify_fit(gb, budget_gb, has_vulkan);
            HfModelCandidate {
                repo_id: repo.to_string(),
                recommended_file: Some(file.to_string()),
                est_size_gb: gb,
                downloads,
                fit_label,
                fits_pc,
                est_tok_s,
            }
        })
        .collect();

    // Prioritize fitting models with highest capacity first
    items.sort_by(|a, b| {
        b.fits_pc.cmp(&a.fits_pc).then_with(|| {
            b.est_size_gb
                .partial_cmp(&a.est_size_gb)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    items
}

pub fn parse_hf_models_json(body: &str, budget_gb: f64, has_vulkan: bool) -> Result<Vec<HfModelCandidate>> {
    let parsed: Vec<HfApiModel> = serde_json::from_str(body).context("Некорректный JSON от Hugging Face")?;
    let items = parsed
        .into_iter()
        .map(|m| {
            let est_size_gb = estimate_params_from_name(&m.id);
            let (fit_label, fits_pc, est_tok_s) = classify_fit(est_size_gb, budget_gb, has_vulkan);
            HfModelCandidate {
                repo_id: m.id,
                recommended_file: None,
                est_size_gb,
                downloads: m.downloads,
                fit_label,
                fits_pc,
                est_tok_s,
            }
        })
        .collect();
    Ok(items)
}

pub fn parse_hf_tree_json(body: &str) -> Result<Vec<HfFileEntry>> {
    let nodes: Vec<HfTreeNode> = serde_json::from_str(body).context("Некорректный список файлов HF")?;
    let mut files: Vec<HfFileEntry> = nodes
        .into_iter()
        .filter(|n| n.node_type == "file" && n.path.to_ascii_lowercase().ends_with(".gguf"))
        .map(|n| HfFileEntry {
            path: n.path,
            size_bytes: n.size,
        })
        .collect();

    // Rank Q4_K_M / Q4_K_S / Q5_K_M / IQ4_XS highest
    files.sort_by_key(|f| {
        let low = f.path.to_ascii_lowercase();
        let rank = if low.contains("q4_k_m") {
            0
        } else if low.contains("iq4_xs") || low.contains("q4_k_s") {
            1
        } else if low.contains("q5_k_m") {
            2
        } else if low.contains("q8_0") {
            3
        } else {
            4
        };
        (rank, f.size_bytes)
    });
    Ok(files)
}

pub fn spawn_hf_search(
    query: String,
    budget_bytes: u64,
    has_vulkan: bool,
    tx: mpsc::UnboundedSender<HfEvent>,
) {
    tokio::spawn(async move {
        let budget_gb = (budget_bytes as f64 / (1024.0 * 1024.0 * 1024.0)).max(1.2);
        let q = query.trim();
        let url = if q.is_empty() {
            "https://huggingface.co/api/models?filter=gguf&sort=downloads&direction=-1&limit=20"
                .to_string()
        } else {
            format!(
                "https://huggingface.co/api/models?filter=gguf&sort=downloads&direction=-1&limit=20&search={}",
                percent_encode(q)
            )
        };

        let client = reqwest::Client::builder()
            .user_agent("llmLab/0.2")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(text) => match parse_hf_models_json(&text, budget_gb, has_vulkan) {
                    Ok(items) => {
                        let _ = tx.send(HfEvent::SearchResults {
                            query: query.clone(),
                            items,
                        });
                    }
                    Err(e) => {
                        let _ = tx.send(HfEvent::Error(format!("HF парсинг: {e}")));
                    }
                },
                Err(e) => {
                    let _ = tx.send(HfEvent::Error(format!("HF чтение ответа: {e}")));
                }
            },
            Ok(resp) => {
                let _ = tx.send(HfEvent::Error(format!("HF HTTP {}", resp.status())));
            }
            Err(e) => {
                let _ = tx.send(HfEvent::Error(format!("Ошибка сети HF: {e}")));
            }
        }
    });
}

pub fn spawn_hf_list_files(repo_id: String, tx: mpsc::UnboundedSender<HfEvent>) {
    tokio::spawn(async move {
        let url = format!("https://huggingface.co/api/models/{repo_id}/tree/main");
        let client = reqwest::Client::builder()
            .user_agent("llmLab/0.2")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => match resp.text().await {
                Ok(text) => match parse_hf_tree_json(&text) {
                    Ok(files) => {
                        let _ = tx.send(HfEvent::RepoFiles { repo_id, files });
                    }
                    Err(e) => {
                        let _ = tx.send(HfEvent::Error(format!("Список файлов: {e}")));
                    }
                },
                Err(e) => {
                    let _ = tx.send(HfEvent::Error(format!("HF чтение файлов: {e}")));
                }
            },
            Ok(resp) => {
                let _ = tx.send(HfEvent::Error(format!("HF files HTTP {}", resp.status())));
            }
            Err(e) => {
                let _ = tx.send(HfEvent::Error(format!("Сеть HF: {e}")));
            }
        }
    });
}

pub fn spawn_hf_download(
    repo_id: String,
    file_path: String,
    models_dir: String,
    tx: mpsc::UnboundedSender<HfEvent>,
) {
    tokio::spawn(async move {
        let res = download_gguf_async(&repo_id, &file_path, &models_dir, &tx).await;
        if let Err(err) = res {
            let _ = tx.send(HfEvent::Error(format!("Скачивание {file_path}: {err}")));
        }
    });
}

async fn download_gguf_async(
    repo_id: &str,
    file_path: &str,
    models_dir: &str,
    tx: &mpsc::UnboundedSender<HfEvent>,
) -> Result<()> {
    let dir = crate::config::resolve_app_subdir(models_dir);
    fs::create_dir_all(&dir)
        .with_context(|| format!("Не удалось создать папку {} (нет прав на запись)", dir.display()))?;

    let clean_name = Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Некорректное имя файла модели"))?
        .to_string();

    let final_path = dir.join(&clean_name);
    let part_path = dir.join(format!("{clean_name}.part"));

    let url = format!("https://huggingface.co/{repo_id}/resolve/main/{file_path}?download=true");
    let client = reqwest::Client::builder()
        .user_agent("llmLab/0.2")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let resp = client.get(&url).send().await.context("Запрос к HF")?;
    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let file = File::create(&part_path).context("Не удалось создать .part файл")?;
    let mut writer = BufWriter::new(file);
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    let started = Instant::now();
    let mut last_emit = Instant::now();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.context("Обрыв потока загрузки")?;
        writer.write_all(&bytes).context("Ошибка записи диска")?;
        downloaded += bytes.len() as u64;

        if last_emit.elapsed().as_millis() >= 150 {
            let secs = started.elapsed().as_secs_f64().max(0.05);
            let speed_mb_s = (downloaded as f64 / (1024.0 * 1024.0)) / secs;
            let _ = tx.send(HfEvent::DownloadProgress {
                file_name: clean_name.clone(),
                downloaded,
                total,
                speed_mb_s,
            });
            last_emit = Instant::now();
        }
    }

    writer.flush().context("Ошибка сброса буфера файла")?;
    drop(writer);

    // Replace any previous identical file cleanly
    if final_path.exists() {
        let _ = fs::remove_file(&final_path);
    }
    fs::rename(&part_path, &final_path).context("Не удалось завершить переименование .gguf")?;

    let _ = tx.send(HfEvent::DownloadDone {
        file_name: clean_name,
        saved_path: final_path,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_hf_search_query() {
        assert_eq!(percent_encode("qwen 2.5 gguf"), "qwen%202.5%20gguf");
    }

    #[test]
    fn ranks_q4_k_m_first_in_tree() {
        let raw = r#"[
            {"type":"file","path":"model-q8_0.gguf","size":3000},
            {"type":"file","path":"README.md","size":120},
            {"type":"file","path":"model-q4_k_m.gguf","size":1500}
        ]"#;
        let files = parse_hf_tree_json(raw).expect("parse tree");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "model-q4_k_m.gguf");
    }

    #[test]
    fn classifies_fit_for_weak_and_strong_pcs() {
        let (label_small, ok_small, _) = classify_fit(2.1, 6.0, false);
        assert!(ok_small);
        assert_eq!(label_small, "[OK · Быстро]");

        let (label_big, ok_big, _) = classify_fit(9.0, 4.0, false);
        assert!(!ok_big);
        assert_eq!(label_big, "[Мало RAM]");
    }
}
