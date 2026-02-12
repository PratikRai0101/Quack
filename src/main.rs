use clap::Parser;
use dotenvy::dotenv;
use std::env;
use tokio::task::JoinHandle;
use tokio::sync::oneshot;

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

    // Start the ask_the_duck task if we have an API key. We wire a oneshot
    // channel to allow graceful cancellation on user quit.
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
    let mut duck_join: Option<JoinHandle<anyhow::Result<()>>> = None;

    if let Some(_key) = api_key.as_deref() {
        // Spawn a background task that would stream AI responses. Here it's
        // a stub and listens for cancellation via the oneshot receiver.
        let _git_ctx_clone = git_ctx.clone();
        duck_join = Some(tokio::spawn(async move {
            // In a real implementation we'd consume groq::ask_the_duck stream
            // and forward chunks to the TUI. For the stub, just await the
            // cancellation signal to simulate long-running work.
            let _ = cancel_rx.await;
            Ok(())
        }));
    }

    // Main blocking loop: draw the TUI and wait for a single keypress from
    // stdin. For this scaffold we'll read a single byte and interpret 'q'
    // or ESC (27) as quit.
    tui.draw(&stderr, "");

    // Read a single byte from stdin synchronously.
    use std::io::{self, Read};
    let mut buf = [0u8; 1];
    let read_res = io::stdin().read(&mut buf);

    let should_quit = match read_res {
        Ok(1) => buf[0] == b'q' || buf[0] == 27,
        _ => false,
    };

    if should_quit {
        // Send cancellation to the background task and wait for it to finish.
        let _ = cancel_tx.send(());
        if let Some(h) = duck_join {
            // Await the join handle; ignore errors but ensure task termination.
            let _ = h.await;
        }
    }

    // Teardown TUI and exit promptly.
    tui.teardown();

    Ok(())
}
