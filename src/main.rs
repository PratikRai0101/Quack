use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;
use std::fs;
use std::process::Command;
use tokio::task::JoinHandle;
use tokio::sync::mpsc;
use futures_util::StreamExt as FuturesStreamExt;
use crossterm::event::{self, Event, KeyCode};
use arboard::Clipboard;
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
#[command(author, version, about, long_about = None)]
struct Args {
    /// Command to replay
    #[arg(long)]
    cmd: Option<String>,
    /// Exit status of the previous command (passed from shell wrapper)
    #[arg(short = 's', long = "status")]
    status: Option<i32>,

    /// Optional positional command tokens (allows wrapper to pass original argv)
    #[arg(last = true)]
    cmd_args: Vec<String>,

    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    /// Install shell integration for quack into the user's shell rc file
    Init,
}

/// Run a minimal TUI-driven loop. Pressing 'q' or Esc will cancel the
/// spawned groq task (if any) and restore the terminal state immediately.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let args = Args::parse();

    // Handle shell integration init subcommand: append wrapper to user's rc
    if let Some(action) = &args.action {
        match action {
            Action::Init => {
                let shell_path = env::var("SHELL").unwrap_or_default();
                let shell_name = std::path::Path::new(&shell_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let home = match dirs::home_dir() {
                    Some(h) => h,
                    None => {
                        eprintln!("Could not determine home directory to install shell integration");
                        return Ok(());
                    }
                };

                let (rc_path, script) = match shell_name.as_str() {
                    "fish" => (
                        home.join(".config/fish/config.fish"),
                        "function quack\n    set -l last_status $status\n    history save\n    command quack --status $last_status $argv\nend\n",
                    ),
                    "zsh" => (
                        home.join(".zshrc"),
                        "quack() {\n    local last_status=$?\n    fc -W\n    command quack --status $last_status \"$@\"\n}\n",
                    ),
                    "bash" => (
                        home.join(".bashrc"),
                        "quack() {\n    local last_status=$?\n    history -a\n    command quack --status $last_status \"$@\"\n}\n",
                    ),
                    other => {
                        eprintln!("Unsupported shell: {}. Supported: zsh, bash, fish", other);
                        return Ok(());
                    }
                };

                // Read existing file content if present
                let existing = std::fs::read_to_string(&rc_path).unwrap_or_default();
                if existing.contains("function quack") || existing.contains("quack() {") {
                    println!("quack integration already present in {}", rc_path.display());
                    return Ok(());
                }

                // Append the script
                use std::fs::OpenOptions;
                use std::io::Write;

                use anyhow::Context;
                let mut f = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&rc_path)
                    .context(format!("Failed to open rc file: {}", rc_path.display()))?;

                writeln!(f, "\n# quack shell integration - added by quack init")?;
                writeln!(f, "{}", script)?;

                println!("Appended quack integration to {}", rc_path.display());
                println!("Restart your shell or source the file to enable 'quack'");
                return Ok(());
            }
        }
    }
    let api_key = env::var("GROQ_API_KEY").ok();

    // Determine whether we have git context available.
    let git_ctx = context::get_git_diff();
    let has_git_context = git_ctx.is_some();

    // Detect OS context: try /etc/os-release PRETTY_NAME, fallback to `uname -a`.
    let os_context = match fs::read_to_string("/etc/os-release") {
        Ok(release) => {
            let mut pretty: Option<String> = None;
            for line in release.lines() {
                if let Some(rest) = line.strip_prefix("PRETTY_NAME=") {
                    // strip surrounding quotes if present
                    let v = rest.trim().trim_matches('"').to_string();
                    pretty = Some(v);
                    break;
                }
            }
            match pretty {
                Some(p) => format!("OS: {}", p),
                None => {
                    // fallback to uname
                    match Command::new("uname").arg("-a").output() {
                        Ok(out) => {
                            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                            format!("OS: {}", s)
                        }
                        Err(_) => "OS: Unknown".to_string(),
                    }
                }
            }
        }
        Err(_) => match Command::new("uname").arg("-a").output() {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                format!("OS: {}", s)
            }
            Err(_) => "OS: Unknown".to_string(),
        },
    };

    // If status was provided by the shell wrapper and it indicates success,
    // exit quietly (graceful silence).
    if let Some(code) = args.status {
        if code == 0 {
            println!("Everything looks ducky! 🦆 (No errors detected)");
            return Ok(());
        }
    }

    // Determine the command to replay. Priority:
    // 1) --cmd string
    // 2) positional cmd_args joined (wrapper may pass $argv)
    // 3) last command from history
    let cmd_to_run = if let Some(cmd) = args.cmd.clone() {
        Some(cmd)
    } else if !args.cmd_args.is_empty() {
        Some(args.cmd_args.join(" "))
    } else {
        None
    };

    let output = if let Some(cmd) = cmd_to_run {
        shell::replay_command(&cmd)?
    } else {
        match shell::get_last_command() {
            Ok(last_cmd) => shell::replay_command(&last_cmd)?,
            Err(_) => {
                eprintln!("Could not read history. Try 'history -a' or use --cmd");
                return Err(anyhow::anyhow!("No command to replay"));
            }
        }
    };

    // Combine stdout and stderr so the UI and AI see both outputs.
    let combined_output = format!("{}\n{}", output.stdout.trim(), output.stderr.trim());

    // Decide whether to launch the TUI: either non-zero exit or any output.
    let should_launch = output.exit_code != 0 || !combined_output.trim().is_empty();

    if !should_launch {
        // Nothing to show; exit quietly after printing any output.
        if !combined_output.trim().is_empty() {
            println!("{}", combined_output);
        }
        return Ok(());
    }

    // Initialize TUI since we have something to display.
    let mut tui = tui::Tui::init()?;

    // App state
    struct AppLocal {
        error_log: String,
        duck_response: String,
        is_streaming: bool,
        has_git_context: bool,
    }

    let mut app = AppLocal {
        error_log: combined_output.clone(),
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
        let combined_clone = combined_output.clone();
        let app_tx_clone = app_tx.clone();
        let os_context_clone = os_context.clone();

        duck_join = Some(tokio::spawn(async move {
        let mut stream = groq::ask_the_duck(&api_key, &combined_clone, git_ctx_clone, os_context_clone);
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

    // Helper: copy string to clipboard. Keep synchronous for simplicity.
    fn copy_to_clipboard(s: String) -> Result<(), String> {
        Clipboard::new()
            .map_err(|e| format!("clipboard init error: {}", e))
            .and_then(|mut cb| cb.set_text(s).map_err(|e| format!("clipboard set error: {}", e)))
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
                    KeyCode::Char('y') => {
                        // Copy the most relevant fix to clipboard.
                        let response = app.duck_response.clone();
                        let mut to_copy: Option<String> = None;

                        // Prefer THE SOLUTION section if present
                        if let Some(idx) = response.to_lowercase().find("the solution") {
                            let rest = &response[idx..];
                            // Try to find a fenced code block inside THE SOLUTION
                            if let Some(start) = rest.find("```") {
                                if let Some(end) = rest[start + 3..].find("```") {
                                    let mut code = rest[start + 3..start + 3 + end].to_string();
                                    // strip leading/trailing newlines
                                    code = code.trim_matches('\n').to_string();
                                    to_copy = Some(code);
                                }
                            }
                            // Fallback to copying the whole solution section
                            if to_copy.is_none() {
                                to_copy = Some(rest.trim().to_string());
                            }
                        } else {
                            // No THE SOLUTION header: try first fenced code block globally
                            if let Some(start) = response.find("```") {
                                if let Some(end) = response[start + 3..].find("```") {
                                    let mut code = response[start + 3..start + 3 + end].to_string();
                                    code = code.trim_matches('\n').to_string();
                                    to_copy = Some(code);
                                }
                            }
                        }

                        // Final fallback: copy entire response
                        if to_copy.is_none() {
                            to_copy = Some(response.trim().to_string());
                        }

                        if let Some(text) = to_copy {
                            match copy_to_clipboard(text.clone()) {
                                Ok(_) => {
                                    // Provide lightweight feedback by appending a short message to the error pane
                                    app.error_log = format!("{}\n\n[Copied fix to clipboard]", app.error_log);
                                }
                                Err(err) => {
                                    app.error_log = format!("{}\n\n[Copy failed: {}]", app.error_log, err);
                                }
                            }
                        }
                    }
                    KeyCode::Char('r') => {
                        // Re-run: spawn a new ask_the_duck task if API key present.
                        // For simplicity, reuse the existing api_key and combined_output
                        // from the surrounding scope by replaying the same flow.
                        // Note: this is a lightweight re-request; it will not cancel the
                        // previous background task in this simple implementation.
                        if let Some(key) = api_key.as_deref() {
                            let git_ctx_clone = git_ctx.clone();
                            let api_key = key.to_string();
                            let combined_clone = combined_output.clone();
                            let app_tx_clone = app_tx.clone();
                            let os_context_clone = os_context.clone();

                            let _ = tokio::spawn(async move {
                                let mut stream = groq::ask_the_duck(&api_key, &combined_clone, git_ctx_clone, os_context_clone);
                                while let Some(msg) = FuturesStreamExt::next(&mut stream).await {
                                    match msg {
                                        Ok(chunk) => {
                                            if !chunk.is_empty() {
                                                let _ = app_tx_clone.send(chunk).await;
                                            }
                                        }
                                        Err(_) => break,
                                    }
                                }
                            });
                        }
                    }
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
