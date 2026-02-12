use anyhow::Result;
use futures_util::stream::{self, BoxStream};

/// ask_the_duck: stub that would stream AI responses from Groq.
pub fn ask_the_duck(_api_key: &str, _error_log: &str, _git_context: Option<String>) -> Result<BoxStream<'static, Result<String>>> {
    // Return an empty stream as a placeholder.
    Ok(stream::empty::<Result<String>>().boxed())
}
