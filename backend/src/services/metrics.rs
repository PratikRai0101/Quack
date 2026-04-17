use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicUsize, Ordering};
use serde::Serialize;

static LLM_STREAM_STARTS: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
static LLM_STREAM_ERRORS: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
static ANALYZE_COUNT: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));
static FOLLOWUP_COUNT: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(0));

pub fn incr_llm_stream_start() { LLM_STREAM_STARTS.fetch_add(1, Ordering::Relaxed); }
pub fn incr_llm_stream_error() { LLM_STREAM_ERRORS.fetch_add(1, Ordering::Relaxed); }
pub fn incr_analyze() { ANALYZE_COUNT.fetch_add(1, Ordering::Relaxed); }
pub fn incr_followup() { FOLLOWUP_COUNT.fetch_add(1, Ordering::Relaxed); }

#[derive(Serialize)]
pub struct MetricsSnapshot {
    pub llm_stream_starts: usize,
    pub llm_stream_errors: usize,
    pub analyze_count: usize,
    pub followup_count: usize,
}

pub fn snapshot() -> MetricsSnapshot {
    MetricsSnapshot {
        llm_stream_starts: LLM_STREAM_STARTS.load(Ordering::Relaxed),
        llm_stream_errors: LLM_STREAM_ERRORS.load(Ordering::Relaxed),
        analyze_count: ANALYZE_COUNT.load(Ordering::Relaxed),
        followup_count: FOLLOWUP_COUNT.load(Ordering::Relaxed),
    }
}
