use anyhow::Result;
use async_stream::stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

pub struct LlmConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

impl LlmConfig {
    pub fn from_env() -> Self {
        let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| std::env::var("PROVIDER").unwrap_or_else(|_| "stub".to_string()));
        let api_key = std::env::var("LLM_API_KEY").ok();
        let model = std::env::var("LLM_MODEL").ok();
        let base_url = std::env::var("LLM_BASE_URL").ok();
        Self { provider, api_key, model, base_url }
    }
}

fn extract_delta_content(v: &Value) -> Option<String> {
    if let Some(s) = v.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("delta"))
        .and_then(|d| d.get("content"))
        .and_then(|x| x.as_str())
    {
        return Some(s.to_string());
    }
    if let Some(s) = v.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("content"))
        .and_then(|x| x.as_str())
    {
        return Some(s.to_string());
    }
    None
}

/// Stubbed followup streaming for frontend/dev.
pub fn stream_followup_stub(session_id: &str, _question: &str) -> impl futures_core::stream::Stream<Item = Result<String>> {
    let id = session_id.to_string();
    stream! {
        yield Ok(format!("### **Follow-up: {}**\n\nLet me think about this...\n", id));
        tokio::time::sleep(Duration::from_millis(120)).await;
        yield Ok("I recommend checking that your types match the expected signature and using explicit casts where needed.\n".to_string());
        tokio::time::sleep(Duration::from_millis(120)).await;
        yield Ok("### **The Solution**\n```rust\n// Example fix\nlet x: i32 = 42;\n```\n".to_string());
    }
}

/// Stream analysis via Groq (or other provider via provider string). Returns
/// a stream of textual chunks (not SSE-framed).
pub fn stream_analysis(config: &LlmConfig, command: &str, stdout: &str, stderr: &str, git_context: Option<String>, os_context: &str) -> impl futures_core::stream::Stream<Item = Result<String>> {
    let cfg = config.clone();
    let command = command.to_string();
    let stdout = stdout.to_string();
    let stderr = stderr.to_string();
    let git = git_context.clone();
    let os = os_context.to_string();

    stream! {
        if cfg.provider == "groq" {
            // Use Groq's OpenAI-compatible endpoint
            let client = Client::new();
            let api_key = cfg.api_key.clone().unwrap_or_default();
            let model = cfg.model.clone().unwrap_or_else(|| "llama-3.3-70b-versatile".to_string());
            let base_url = cfg.base_url.clone().unwrap_or_else(|| "https://api.groq.com/openai/v1/chat/completions".to_string());

            let mut user_content = format!("Command: {}\n\nStderr:\n{}\n\nStdout:\n{}\n", command, stderr, stdout);
            if let Some(ctx) = git {
                if !ctx.is_empty() {
                    user_content.push_str("\n\nRECENT CODE CHANGES:\n");
                    user_content.push_str(&ctx);
                }
            }

            let system_prompt = format!("Expert System Debugger. OS: {}", os);

            let body = serde_json::json!({
                "model": model,
                "stream": true,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_content}
                ]
            });

            let resp = match client.post(&base_url).bearer_auth(api_key).json(&body).send().await {
                Ok(r) => r,
                Err(e) => { let _ = yield Err(anyhow::anyhow!(e)); return; }
            };

            let mut stream = resp.bytes_stream();
            let mut buf = Vec::new();

            while let Some(item) = stream.next().await {
                match item {
                    Ok(bytes) => {
                        buf.extend_from_slice(&bytes);
                        while let Some(pos) = find_double_newline(&buf) {
                            let chunk_bytes = buf.drain(..pos+2).collect::<Vec<u8>>();
                            if let Ok(s) = String::from_utf8(chunk_bytes) {
                                for line in s.lines() {
                                    let line = line.trim();
                                    if line.is_empty() { continue; }
                                    let payload = if let Some(rest) = line.strip_prefix("data: ") { rest } else { line };
                                    if payload == "[DONE]" { continue; }
                                    if let Ok(v) = serde_json::from_str::<Value>(payload) {
                                        if let Some(text) = extract_delta_content(&v) {
                                            if yield Ok(text).is_err() { return; }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => { let _ = yield Err(anyhow::anyhow!(e)); return; }
                }
            }
        } else {
            // Fallback to stubbed analysis
            let _ = yield Ok(format!("### **Analysis: {}**\n\nThis is a simulated analysis (stub) for frontend development.\n", command));
            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = yield Ok("### **The Glitch**\nA simulated compiler error occurred.\n".to_string());
            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = yield Ok("### **The Solution**\n```rust\nlet x: i32 = 42;\n```\n".to_string());
        }
    }
}

fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
}

/// Stream a followup: use provider if configured, else stub.
pub fn stream_followup(config: &LlmConfig, session_id: &str, question: &str) -> impl futures_core::stream::Stream<Item = Result<String>> {
    if config.provider == "groq" {
        // For now reuse stream_analysis with a small wrapper
        stream_analysis(config, question, "", "", None, "")
    } else {
        stream_followup_stub(session_id, question)
    }
}
