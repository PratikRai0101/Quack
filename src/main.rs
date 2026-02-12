use clap::Parser;
use dotenvy::dotenv;
use std::env;
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use futures_util::StreamExt as FuturesStreamExt;

mod groq;
mod tui;
mod context;
mod shell;

#[derive(Parser)]
struct Args {
    /// Command to replay
    #[arg(long)]
    cmd: Option<String>,
}

/// Run a minimal TUI-driven loop. Pressing 'q' or Esc will cancel the
/// spawned groq task (if any) and restore the terminal state immediately.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();
    let api_key = env::var("GROQ_API_KEY").ok();

    // Determine whether we have git context available.
    let git_ctx = context::get_git_diff();
    let has_git_context = git_ctx.is_some();

    // Initialize TUI with the git context flag.
    let mut tui = tui::Tui::init(has_git_context)?;

    // Placeholder: if a command is provided we'd replay it and capture stderr.
    let stderr = if let Some(cmd) = args.cmd {
        let out = shell::replay_command(&cmd)?;
        out.stderr
    } else {
        String::new()
    };

    // Start the ask_the_duck task if we have an API key. This spawns the
    // real groq::ask_the_duck stream and forwards chunks to the main loop
    // via an mpsc channel so the UI can be updated progressively.
    let (app_tx, mut app_rx) = mpsc::channel::<String>(128);
    let mut duck_join: Option<JoinHandle<()>> = None;

    if let Some(key) = api_key.as_deref() {
        let git_ctx_clone = git_ctx.clone();
        let api_key = key.to_string();
        let stderr_clone = stderr.clone();
        let app_tx_clone = app_tx.clone();

        duck_join = Some(tokio::spawn(async move {
            let mut stream = groq::ask_the_duck(&api_key, &stderr_clone, git_ctx_clone);
            while let Some(msg) = FuturesStreamExt::next(&mut stream).await {
                match msg {
                    Ok(chunk) => {
                        // Some chunks may be empty markers; forward non-empty
                        if !chunk.is_empty() {
                            let _ = app_tx_clone.send(chunk).await;
                        }
                    }
                    Err(_e) => {
                        // For v0.1 keep it simple: stop on error.
                        break;
                    }
                }
            }
        }));
    }

    // Main blocking loop: draw the TUI and wait for a single keypress from
    // stdin. For this scaffold we'll read a single byte and interpret 'q'
    // or ESC (27) as quit.
    tui.draw(&stderr, "");

    // Event loop: redraw UI on incoming chunks; also allow quitting by
    // reading one byte from stdin and interpreting 'q' or Esc as quit.
    let mut app_duck_response = String::new();

    // spawn a blocking task to read a single keypress so we don't block
    // the tokio runtime on std::io.
    // We'll repeatedly spawn the blocking key reader inside the loop so each
    // iteration has a fresh handle (avoids moving the JoinHandle).
    loop {
        let key_read = tokio::task::spawn_blocking(|| {
            use std::io::{self, Read};
            let mut buf = [0u8; 1];
            let _ = io::stdin().read(&mut buf);
            buf[0]
        });
        tokio::select! {
            // New AI chunk
            Some(chunk) = app_rx.recv() => {
                app_duck_response.push_str(&chunk);
                tui.draw(&stderr, &app_duck_response);
            }
            // Keypress arrived
            key = key_read => {
                if let Ok(b) = key {
                    if b == b'q' || b == 27 {
                        break;
                    }
                }
                // no quit -> continue to receive AI chunks
            }
        }
    }

    // On quit, ensure the background task finishes gracefully.
    if let Some(h) = duck_join {
        let _ = h.await;
    }

    // Teardown TUI and exit promptly.
    tui.teardown();

    Ok(())
}
