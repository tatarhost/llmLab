use crate::hf::percent_encode;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    AppBuilder,
    WebResearcher,
    FastAssistant,
}

impl AgentRole {

    pub fn label(self) -> &'static str {
        match self {
            Self::AppBuilder => "Агент-разработчик приложений",
            Self::WebResearcher => "Агент веб-поиска и анализа",
            Self::FastAssistant => "Быстрый системный чат",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::AppBuilder => "Builder Agent",
            Self::WebResearcher => "Web Agent",
            Self::FastAssistant => "Fast Chat",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::AppBuilder => Self::WebResearcher,
            Self::WebResearcher => Self::FastAssistant,
            Self::FastAssistant => Self::AppBuilder,
        }
    }

    pub fn role_instructions(self) -> &'static str {
        match self {
            Self::AppBuilder => {
                "You are an autonomous PC App Builder agent inside llmLab. Build clean, working applications inside ./workspace using tools step-by-step, verify builds/tests with run_command, and report concise results."
            }
            Self::WebResearcher => {
                "You are a Web Research agent inside llmLab. Use web_search and fetch_url to gather accurate up-to-date facts, libraries, or docs before answering concisely with sources."
            }
            Self::FastAssistant => {
                "You are a fast, concise local AI assistant running in llmLab."
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillFlags {
    pub web_search: bool,
    pub file_builder: bool,
    pub shell_exec: bool,
    pub auto_approve_tools: bool,
}

impl Default for SkillFlags {
    fn default() -> Self {
        Self {
            web_search: true,
            file_builder: true,
            shell_exec: false,
            auto_approve_tools: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    pub name: String,
    pub query: String,
    pub path: String,
    pub content: String,
    pub command: String,
}

pub fn build_agent_system_prompt(base_prompt: &str, role: AgentRole, skills: &SkillFlags) -> String {
    let mut prompt = format!("{base_prompt}\n{}\n", role.role_instructions());

    let mut active_tools = Vec::new();
    if skills.web_search {
        active_tools.push(r#"- web_search: {"tool":"web_search","query":"search terms"}"#);
        active_tools.push(r#"- fetch_url: {"tool":"fetch_url","query":"https://..."}"#);
    }
    if skills.file_builder {
        active_tools.push(r#"- list_files: {"tool":"list_files","path":"."}"#);
        active_tools.push(r#"- read_file: {"tool":"read_file","path":"app.py"}"#);
        active_tools.push(
            r#"- write_file: {"tool":"write_file","path":"app.py","content":"file contents"}"#,
        );
    }
    if skills.shell_exec {
        active_tools.push(r#"- run_command: {"tool":"run_command","command":"python app.py"}"#);
    }

    if !active_tools.is_empty() {
        prompt.push_str(
            "\nWhen you need an external action, output ONLY one tool block using:\n<<<TOOL\n{\"tool\":\"...\",...}\n>>>\nAvailable tools:\n",
        );
        prompt.push_str(&active_tools.join("\n"));
    }

    prompt
}

/// Parses a structured tool invocation emitted by the LLM (`<<<TOOL ... >>>`).
pub fn extract_tool_call(text: &str) -> Option<ToolCall> {
    let start_marker = "<<<TOOL";
    let end_marker = ">>>";
    let start = text.find(start_marker)?;
    let after_start = &text[start + start_marker.len()..];
    let end = after_start.find(end_marker)?;
    let json_raw = after_start[..end].trim();

    let v: serde_json::Value = serde_json::from_str(json_raw).ok()?;
    let name = v.get("tool")?.as_str()?.trim().to_string();
    Some(ToolCall {
        name,
        query: v
            .get("query")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        path: v
            .get("path")
            .and_then(|s| s.as_str())
            .unwrap_or(".")
            .to_string(),
        content: v
            .get("content")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        command: v
            .get("command")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

/// Ensures file tools remain strictly sandboxed inside `./workspace`.
pub fn resolve_workspace_path(workspace_root: &Path, rel: &str) -> Result<PathBuf> {
    let trimmed = rel.trim().trim_start_matches(['/', '\\']);
    let candidate = Path::new(if trimmed.is_empty() { "." } else { trimmed });
    if candidate.is_absolute() {
        return Err(anyhow!("Абсолютные пути запрещены в песочнице агента"));
    }
    for comp in candidate.components() {
        if matches!(comp, Component::ParentDir | Component::RootDir | Component::Prefix(_)) {
            return Err(anyhow!("Выход за пределы ./workspace запрещён"));
        }
    }
    Ok(workspace_root.join(candidate))
}

pub async fn execute_tool(
    call: &ToolCall,
    workspace_root: &Path,
    skills: &SkillFlags,
) -> String {
    let _ = fs::create_dir_all(workspace_root);

    match call.name.as_str() {
        "web_search" => {
            if !skills.web_search {
                return "Навык web_search отключён в [Скиллы].".to_string();
            }
            match run_web_search(&call.query).await {
                Ok(out) => out,
                Err(e) => format!("Ошибка веб-поиска: {e}"),
            }
        }
        "fetch_url" => {
            if !skills.web_search {
                return "Навык web_search отключён в [Скиллы].".to_string();
            }
            match fetch_clean_url(&call.query).await {
                Ok(out) => out,
                Err(e) => format!("Ошибка загрузки URL: {e}"),
            }
        }
        "list_files" => {
            if !skills.file_builder {
                return "Навык file_builder отключён.".to_string();
            }
            match list_workspace_files(workspace_root, &call.path) {
                Ok(out) => out,
                Err(e) => format!("Ошибка list_files: {e}"),
            }
        }
        "read_file" => {
            if !skills.file_builder {
                return "Навык file_builder отключён.".to_string();
            }
            match resolve_workspace_path(workspace_root, &call.path)
                .and_then(|p| fs::read_to_string(p).context("Не удалось прочитать файл"))
            {
                Ok(txt) => txt.chars().take(3500).collect(),
                Err(e) => format!("Ошибка read_file: {e}"),
            }
        }
        "write_file" => {
            if !skills.file_builder {
                return "Навык file_builder отключён.".to_string();
            }
            match write_workspace_file(workspace_root, &call.path, &call.content) {
                Ok(msg) => msg,
                Err(e) => format!("Ошибка write_file: {e}"),
            }
        }
        "run_command" => {
            if !skills.shell_exec {
                return "Навык shell_exec отключён (включите в [4] Скиллы).".to_string();
            }
            match run_workspace_shell(workspace_root, &call.command).await {
                Ok(out) => out,
                Err(e) => format!("Ошибка run_command: {e}"),
            }
        }
        other => format!("Неизвестный инструмент '{other}'"),
    }
}

fn list_workspace_files(workspace_root: &Path, rel: &str) -> Result<String> {
    let target = resolve_workspace_path(workspace_root, rel)?;
    let entries = fs::read_dir(&target).context("Папка не найдена")?;
    let mut lines = Vec::new();
    for entry in entries.flatten().take(40) {
        let name = entry.file_name().to_string_lossy().to_string();
        let kind = if entry.path().is_dir() { "dir" } else { "file" };
        lines.push(format!("[{kind}] {name}"));
    }
    if lines.is_empty() {
        Ok("Папка ./workspace пуста.".to_string())
    } else {
        Ok(lines.join("\n"))
    }
}

fn write_workspace_file(workspace_root: &Path, rel: &str, content: &str) -> Result<String> {
    let target = resolve_workspace_path(workspace_root, rel)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).context("Не удалось создать каталог")?;
    }
    fs::write(&target, content).context("Не удалось записать файл")?;
    Ok(format!(
        "Файл сохранён: {} ({} байт)",
        target.display(),
        content.len()
    ))
}

async fn run_workspace_shell(workspace_root: &Path, command: &str) -> Result<String> {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", command]);
        c
    };

    cmd.current_dir(workspace_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let out = cmd.output().await.context("Не удалось выполнить команду")?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!(
        "exit={}\nstdout:\n{}\nstderr:\n{}",
        out.status.code().unwrap_or(-1),
        stdout.chars().take(1800).collect::<String>(),
        stderr.chars().take(700).collect::<String>()
    );
    Ok(combined)
}

async fn run_web_search(query: &str) -> Result<String> {
    let clean = query.trim();
    if clean.is_empty() {
        return Err(anyhow!("Пустой поисковый запрос"));
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) llmLab/0.2")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let html_url = format!("https://html.duckduckgo.com/html/?q={}", percent_encode(clean));
    if let Ok(resp) = client.get(&html_url).send().await {
        if resp.status().is_success() {
            if let Ok(body) = resp.text().await {
                let parsed = extract_ddg_snippets(&body);
                if !parsed.is_empty() {
                    return Ok(parsed.join("\n"));
                }
            }
        }
    }

    let api_url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        percent_encode(clean)
    );
    let resp = client.get(&api_url).send().await.context("DuckDuckGo API")?;
    let v: serde_json::Value = resp.json().await.context("JSON ответ поиска")?;
    let mut lines = Vec::new();
    if let Some(abs) = v.get("AbstractText").and_then(|s| s.as_str()) {
        if !abs.is_empty() {
            lines.push(format!("Summary: {abs}"));
        }
    }
    if let Some(topics) = v.get("RelatedTopics").and_then(|t| t.as_array()) {
        for item in topics.iter().take(5) {
            if let Some(text) = item.get("Text").and_then(|s| s.as_str()) {
                let url = item.get("FirstURL").and_then(|s| s.as_str()).unwrap_or("");
                lines.push(format!("- {text} ({url})"));
            }
        }
    }
    if lines.is_empty() {
        Ok("По запросу ничего не найдено.".to_string())
    } else {
        Ok(lines.join("\n"))
    }
}

async fn fetch_clean_url(url: &str) -> Result<String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(anyhow!("URL должен начинаться с http:// или https://"));
    }
    let client = reqwest::Client::builder()
        .user_agent("llmLab/0.2")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let resp = client.get(url).send().await.context("Запрос страницы")?;
    let raw = resp.text().await.context("Чтение текста страницы")?;
    Ok(strip_html_tags(&raw).chars().take(3500).collect())
}

pub fn strip_html_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len() / 2);
    let mut inside_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                out.push(' ');
            }
            _ if !inside_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn extract_ddg_snippets(html: &str) -> Vec<String> {
    let mut results = Vec::new();
    let marker = "class=\"result__snippet\"";
    let mut rest = html;
    while let Some(pos) = rest.find(marker) {
        rest = &rest[pos + marker.len()..];
        let Some(gt) = rest.find('>') else { break };
        let after = &rest[gt + 1..];
        let Some(end) = after.find("</a>") else { break };
        let clean = strip_html_tags(&after[..end]);
        if !clean.is_empty() {
            results.push(format!("- {clean}"));
        }
        if results.len() >= 5 {
            break;
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_structured_tool_block() {
        let sample = "Thinking...\n<<<TOOL\n{\"tool\":\"web_search\",\"query\":\"rust ratatui\"}\n>>>";
        let call = extract_tool_call(sample).expect("parse tool call");
        assert_eq!(call.name, "web_search");
        assert_eq!(call.query, "rust ratatui");
    }

    #[test]
    fn blocks_path_traversal_outside_workspace() {
        let root = Path::new("./workspace");
        assert!(resolve_workspace_path(root, "src/main.py").is_ok());
        assert!(resolve_workspace_path(root, "../secret.txt").is_err());
    }
}
