use anyhow::Result;
use futures_util::stream::{self, BoxStream};
use futures_util::StreamExt;

/// ask_the_duck: stub that would stream AI responses from Groq.
///
/// Kept intentionally; it's referenced by design in the project and will be
/// used in later commits to provide the streaming implementation.
pub fn ask_the_duck(
    _api_key: &str,
    _error_log: &str,
    _git_context: Option<String>,
) -> Result<BoxStream<'static, Result<String>>> {
    // Return an empty stream as a placeholder.
    Ok(stream::empty::<Result<String>>().boxed())
}

#[allow(dead_code)]
fn _keep_ask_the_duck_referenced() {
    // tiny helper to silence the dead_code lint for the stub while keeping
    // the symbol available for future use.
    let _ = ask_the_duck("", "", None);
}
