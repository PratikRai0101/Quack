use anyhow::Result;
use async_stream::stream;
use std::time::Duration;

/// Lightweight LLM service abstraction for Quack v2.
///
/// For now we provide a stub streaming implementation (QUACK_STUB_LLM) and
/// a placeholder for real provider integration.

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

/// Stream a stubbed followup response. Yields several chunks (strings) which
/// are the logical content pieces (not SSE-encoded). Callers are responsible
/// for converting to SSE framing.
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

// Placeholder for real streaming provider integration. Implementations should
// return a Stream<Item = Result<String>> that yields content chunks as they
// arrive from the model provider.
// pub fn stream_followup_real(config: &LlmConfig, session_id: &str, question: &str) -> impl Stream<Item = Result<String>> { ... }
