use clap::Parser;
use dotenvy::dotenv;
use std::env;
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use futures_util::StreamExt as FuturesStreamExt;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

mod groq;
mod tui;
mod context;
mod shell;

// App facade passed to the TUI draw function
pub struct App {
    pub error_log: String,
    pub duck_response: String,
    pub is_streaming: bool,
    pub has_git_context: bool,
}

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
    let api_key = env::var("GROQ_API_KEY").ok()

    // Determine whether we have git context available.
    let git_ctx = context::get_git_diff();
    let has_git_context = git_ctx.is_some();

    // Initialize TUI.
    let mut tui = tui::Tui::init()?;

    // Determine the command to replay. If none provided, attempt to read
    // the last command from the user's shell history (bash/zsh/fish).
    let stderr = if let Some(cmd) = args.cmd {
        let out = shell::replay_command(&cmd)?;
        out.stderr
    } else {
        match shell::get_last_command() {
            Ok(last_cmd) => {
                let out = shell::replay_command(&last_cmd)?;
                out.stderr
            }
            Err(_) => {
                eprintln!("Could not read history. Try 'history -a' or use --cmd");
                return Err(anyhow::anyhow!("No command to replay"));
            }
        }
    };

    // App state
    struct AppLocal {
        error_log: String,
        duck_response: String,
        is_streaming: bool,
        has_git_context: bool,
    }

    let mut app = AppLocal {
        error_log: stderr.clone(),
        duck_response: String::new(),
        is_streaming: false,
        has_git_context,
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

    // Main TUI event loop: poll for key events and drain AI chunks.
    loop {
        // Drain incoming AI chunks first
        while let Ok(chunk) = app_rx.try_recv() {
            if !chunk.is_empty() {
                app.duck_response.push_str(&chunk);
                app.is_streaming = true;
            }
        }

        // Draw UI
        // Build a lightweight App facade expected by the TUI draw function
        let app_for_draw = crate::App {
            error_log: app.error_log.clone(),
            duck_response: app.duck_response.clone(),
            is_streaming: app.is_streaming,
            has_git_context: app.has_git_context,
        };
        let _ = tui.draw(&app_for_draw);

        // Poll for input events with a short timeout for responsiveness
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    // On quit, ensure the background task finishes gracefully.
    if let Some(h) = duck_join {
        let _ = h.await;
    }

    // Teardown TUI and exit promptly.
    let _ = tui.exit();

    Ok(())
}
