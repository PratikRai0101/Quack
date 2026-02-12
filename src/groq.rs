use anyhow::Result;
use futures_util::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// ask_the_duck: start an async task that streams Groq responses and
/// returns a ReceiverStream over which textual chunks will be yielded.
pub fn ask_the_duck(api_key: &str, error_log: &str, git_context: Option<String>, os_context: String) -> impl futures_util::Stream<Item = Result<String>> {
    let (tx, rx) = mpsc::channel::<Result<String>>(32);

    let api_key = api_key.to_string();
    let error_log = error_log.to_string();

    tokio::spawn(async move {
        let client = reqwest::Client::new();

        let mut user_content = error_log.clone();
        if let Some(ctx) = git_context {
            if !ctx.is_empty() {
                user_content.push_str("\n\nRECENT CODE CHANGES:\n");
                user_content.push_str(&ctx);
            }
        }

        let system_prompt = format!(
            "Expert CLI debugging assistant running on {}. Your goal is to solve the user's error instantly. Do not give generic advice like 'check the manual' or 'read the help page'.\n\nFollow this exact structure:\n1) One short punchy sentence explaining the root cause.\n2) A FIXED command wrapped in a markdown fenced code block (```bash ... ```).\n3) If a package is likely missing, provide the specific install command for the detected OS (use the native package manager).\nKeep responses concise and immediately actionable.",
            os_context
        );

        let body = serde_json::json!({
            "model": "llama-3.3-70b-versatile",
            "stream": true,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_content}
            ]
        });

        let resp = match client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(&api_key)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(Err(anyhow::anyhow!(e))).await;
                return;
            }
        };

        let mut stream = resp.bytes_stream();
        let mut buf = Vec::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    buf.extend_from_slice(&bytes);

                    // process complete events separated by double newline
                    while let Some(pos) = find_double_newline(&buf) {
                        let chunk_bytes = buf.drain(..pos + 2).collect::<Vec<u8>>();
                        if let Ok(s) = String::from_utf8(chunk_bytes) {
                            for line in s.lines() {
                                let line = line.trim();
                                if line.is_empty() {
                                    continue;
                                }
                                let payload = if let Some(rest) = line.strip_prefix("data: ") {
                                    rest
                                } else {
                                    line
                                };
                                if payload == "[DONE]" {
                                    let _ = tx.send(Ok(String::new())).await;
                                    continue;
                                }
                                if let Ok(v) = serde_json::from_str::<Value>(payload) {
                                    if let Some(text) = extract_delta_content(&v) {
                                        if tx.send(Ok(text)).await.is_err() {
                                            // receiver closed
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!(e))).await;
                    break;
                }
            }
        }
    });

    ReceiverStream::new(rx)
}

fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
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
    if let Some(s) = v.pointer("/choices/0/delta/delta/content").and_then(|x| x.as_str()) {
        return Some(s.to_string());
    }
    None
}
