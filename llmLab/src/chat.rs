use crate::config::{AppConfig, EngineKind};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct SpeedHud {
    pub tok_per_sec: f64,
    pub ttft_ms: Option<u64>,
    pub tokens_generated: usize,
    pub is_streaming: bool,
    pub backend_label: String,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Delta(String),
    Stats {
        tok_per_sec: f64,
        ttft_ms: Option<u64>,
        tokens_generated: usize,
    },
    Done,
    Error(String),
}

#[derive(Debug, Serialize)]
struct ChatRequestPayload {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct SseChunk {
    choices: Vec<SseChoice>,
}

#[derive(Debug, Deserialize)]
struct SseChoice {
    delta: Option<SseDelta>,
}

#[derive(Debug, Deserialize)]
struct SseDelta {
    content: Option<String>,
}

/// Extracts content delta from a single OpenAI-compatible SSE `data: ...` line.
pub fn parse_sse_data_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let payload = trimmed.strip_prefix("data:")?.trim();
    if payload.is_empty() || payload == "[DONE]" {
        return None;
    }
    let chunk: SseChunk = serde_json::from_str(payload).ok()?;
    chunk
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.delta)
        .and_then(|d| d.content)
}

pub fn spawn_chat_stream(
    cfg: AppConfig,
    history: Vec<ChatMessage>,
    tx: mpsc::UnboundedSender<StreamEvent>,
) {
    tokio::spawn(async move {
        let (endpoint, model, api_key) = match cfg.engine {
            EngineKind::CloudApi => {
                let base = cfg.cloud_base_url.trim_end_matches('/');
                let key = std::env::var(&cfg.cloud_api_key_env).ok();
                (
                    format!("{base}/chat/completions"),
                    cfg.cloud_model.clone(),
                    key,
                )
            }
            _ => (
                format!("http://127.0.0.1:{}/v1/chat/completions", cfg.server_port),
                "local-model".to_string(),
                None,
            ),
        };

        if cfg.engine == EngineKind::CloudApi && api_key.as_deref().unwrap_or("").is_empty() {
            let _ = tx.send(StreamEvent::Error(format!(
                "Установите переменную окружения {} для облачного API ({}).",
                cfg.cloud_api_key_env,
                cfg.cloud_provider.label()
            )));
            return;
        }

        let mut messages = vec![ChatMessage {
            role: "system".to_string(),
            content: cfg.system_prompt.clone(),
        }];
        messages.extend(history);

        let payload = ChatRequestPayload {
            model,
            messages,
            stream: true,
            temperature: 0.7,
        };

        let client = reqwest::Client::new();
        let mut req = client.post(&endpoint).json(&payload);
        if let Some(key) = api_key {
            req = req.bearer_auth(key);
        }

        let started_at = Instant::now();
        let resp = match req.send().await {
            Ok(r) => r,
            Err(err) => {
                let _ = tx.send(StreamEvent::Error(format!("Ошибка подключения: {err}")));
                return;
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            let _ = tx.send(StreamEvent::Error(format!(
                "HTTP {status}: {}",
                body.chars().take(180).collect::<String>()
            )));
            return;
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut first_token_at: Option<Instant> = None;
        let mut ttft_ms: Option<u64> = None;
        let mut token_chunks = 0usize;

        while let Some(chunk_res) = stream.next().await {
            let Ok(bytes) = chunk_res else {
                let _ = tx.send(StreamEvent::Error("Поток прерван".to_string()));
                return;
            };
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].to_string();
                buffer.drain(..=pos);

                if let Some(text) = parse_sse_data_line(&line) {
                    if first_token_at.is_none() {
                        let now = Instant::now();
                        first_token_at = Some(now);
                        ttft_ms = Some(now.duration_since(started_at).as_millis() as u64);
                    }
                    token_chunks += 1;
                    let gen_secs = first_token_at
                        .map(|t| t.elapsed().as_secs_f64().max(0.001))
                        .unwrap_or(0.001);
                    let tok_per_sec = token_chunks as f64 / gen_secs;

                    let _ = tx.send(StreamEvent::Delta(text));
                    let _ = tx.send(StreamEvent::Stats {
                        tok_per_sec,
                        ttft_ms,
                        tokens_generated: token_chunks,
                    });
                }
            }
        }

        let _ = tx.send(StreamEvent::Done);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openai_sse_delta() {
        let line = r#"data: {"choices":[{"delta":{"content":"Привет"}}]}"#;
        assert_eq!(parse_sse_data_line(line).as_deref(), Some("Привет"));
        assert_eq!(parse_sse_data_line("data: [DONE]"), None);
        assert_eq!(parse_sse_data_line(": keepalive"), None);
    }
}
