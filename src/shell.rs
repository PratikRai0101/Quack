use anyhow::Context;
use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct CommandOutput {
    // We only read stderr in the current v0.1 CLI flow; keep other fields
    // but prefix unused with underscore to silence warnings.
    pub _stdout: String,
    pub stderr: String,
    pub _exit_code: i32,
}

pub fn replay_command(_command: &str) -> Result<CommandOutput> {
    // Placeholder: real implementation executes the shell command and captures output.
    Ok(CommandOutput {
        _stdout: String::new(),
        stderr: String::new(),
        _exit_code: 0,
    })
}

/// Try to read the last command from the user's shell history.
/// Supports zsh, bash and fish history files.
pub fn get_last_command() -> Result<String> {
    // Determine shell from $SHELL
    let shell_path = env::var("SHELL").unwrap_or_default();
    let shell_name = std::path::Path::new(&shell_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Determine history file path. Prefer HISTFILE env var when present.
    let histfile_env = env::var("HISTFILE").ok();
    let home =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    let history_path: PathBuf = match shell_name.as_str() {
        "zsh" => histfile_env
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".zsh_history")),
        "bash" => histfile_env
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".bash_history")),
        "fish" => histfile_env
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share/fish/fish_history")),
        _ => {
            // Fallback to bash history in unknown shells
            histfile_env
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".bash_history"))
        }
    };

    let contents = fs::read_to_string(&history_path)
        .with_context(|| format!("Failed to read history file: {}", history_path.display()))?;

    // Iterate lines from the end and find the last meaningful entry.
    for line in contents.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match shell_name.as_str() {
            "zsh" => {
                // zsh history lines: ": 167899:0;cargo run"
                if let Some(pos) = line.find(';') {
                    let cmd = line[pos + 1..].trim();
                    if !cmd.is_empty() {
                        return Ok(cmd.to_string());
                    }
                } else if !line.starts_with(':') {
                    // fallback: use the whole line if it doesn't look like metadata
                    return Ok(line.to_string());
                }
            }
            "fish" => {
                // fish history often has lines like "- cmd: cargo run"
                if let Some(cmd) = line.strip_prefix("- cmd: ") {
                    let cmd = cmd.trim();
                    if !cmd.is_empty() {
                        return Ok(cmd.to_string());
                    }
                } else if let Some(cmd) = line.strip_prefix("cmd: ") {
                    let cmd = cmd.trim();
                    if !cmd.is_empty() {
                        return Ok(cmd.to_string());
                    }
                } else {
                    // fallback: use the whole line
                    return Ok(line.to_string());
                }
            }
            "bash" | _ => {
                // bash history may contain timestamp lines starting with '#'
                if line.starts_with('#') {
                    continue;
                }
                return Ok(line.to_string());
            }
        }
    }

    Err(anyhow::anyhow!("No command found in history"))
}
