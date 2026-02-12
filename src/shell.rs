use anyhow::Context;
use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct CommandOutput {
    // We only read stderr in the current v0.1 CLI flow; keep other fields
    // but prefix unused with underscore to silence warnings.
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub fn replay_command(command: &str) -> Result<CommandOutput> {
    // Use the user's shell to evaluate the command string so quoting and
    // flags are parsed as the shell would. Default to `sh` when SHELL
    // env var is not present.
    let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
    let output = std::process::Command::new(shell)
        .arg("-c")
        .arg(command)
        .output()
        .with_context(|| format!("Failed to execute command via shell: {}", command))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(CommandOutput {
        stdout,
        stderr,
        exit_code,
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

    // Iterate lines from the end and find the last meaningful entry using parser.
    // Apply a filter to skip commands that are part of the CLI integration
    // itself (so we don't re-run `quack`/`duck`/history/fc entries).
    let forbidden = ["quack", "duck", "history", "fc"];

    for line in contents.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(cmd) = parse_history_line(line, shell_name.as_str()) {
            // get first word of the parsed command to compare against forbidden prefixes
            let first = cmd.split_whitespace().next().unwrap_or("").to_lowercase();
            if forbidden.iter().any(|f| *f == first) {
                // skip this entry and continue searching backwards
                continue;
            }

            return Ok(cmd);
        }
        // else continue scanning previous lines (handles fish 'when:' lines etc.)
    }

    Err(anyhow::anyhow!("No command found in history"))
}

/// Parse a single history line for a given shell type and return the command
/// if the line represents a runnable command. `shell_type` should be lowercased
/// values like "zsh", "bash", or "fish". Returns None when the line should
/// be skipped (timestamps, empty, metadata-only lines).
pub fn parse_history_line(line: &str, shell_type: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    match shell_type {
        "zsh" => {
            if let Some(pos) = line.find(';') {
                let cmd = line[pos + 1..].trim();
                if !cmd.is_empty() {
                    return Some(cmd.to_string());
                }
            } else if !line.starts_with(':') {
                return Some(line.to_string());
            }
            None
        }
        "fish" => {
            // Fish history is structured; only accept explicit command lines.
            if let Some(cmd) = line.strip_prefix("- cmd: ") {
                let cmd = cmd.trim();
                if !cmd.is_empty() {
                    return Some(cmd.to_string());
                }
                return None;
            }
            if let Some(cmd) = line.strip_prefix("cmd: ") {
                let cmd = cmd.trim();
                if !cmd.is_empty() {
                    return Some(cmd.to_string());
                }
                return None;
            }
            // If the line isn't an explicit command entry (e.g. 'when:' or other
            // metadata), skip it so we don't treat timestamps as commands.
            None
        }
        "bash" | _ => {
            if line.starts_with('#') {
                return None;
            }
            Some(line.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_history_line;

    #[test]
    fn test_zsh_line() {
        let input = ": 167899:0;cargo run --release";
        let out = parse_history_line(input, "zsh");
        assert_eq!(out.as_deref(), Some("cargo run --release"));
    }

    #[test]
    fn test_bash_simple() {
        let input = "ls -la";
        let out = parse_history_line(input, "bash");
        assert_eq!(out.as_deref(), Some("ls -la"));
    }

    #[test]
    fn test_bash_timestamp() {
        let input = "#167899";
        let out = parse_history_line(input, "bash");
        assert_eq!(out, None);
    }

    #[test]
    fn test_fish_with_prefix() {
        let input = "- cmd: cargo build";
        let out = parse_history_line(input, "fish");
        assert_eq!(out.as_deref(), Some("cargo build"));
    }

    #[test]
    fn test_fish_clean() {
        let input = "cargo check";
        let out = parse_history_line(input, "fish");
        // For fish, only explicit command entries are accepted (e.g. '- cmd: ...')
        // plain lines should be ignored.
        assert_eq!(out, None);
    }
}
