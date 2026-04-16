use std::process::Command;
use std::path::Path;
use std::fs;

/// Return recent git diff (git diff HEAD) if available.
pub fn get_git_diff() -> Option<String> {
    match Command::new("git").arg("diff").arg("HEAD").output() {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).into_owned();
            Some(s)
        }
        _ => None,
    }
}

/// Detect a human-friendly OS string. Try /etc/os-release PRETTY_NAME,
/// fallback to `uname -a` if not available.
pub fn detect_os() -> String {
    // Try /etc/os-release first
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("PRETTY_NAME=") {
                // Trim quotes if present
                let val = rest.trim().trim_matches('"').to_string();
                if !val.is_empty() {
                    return val;
                }
            }
        }
    }

    // Fallback to uname -a
    if let Ok(output) = Command::new("uname").arg("-a").output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).to_string();
        }
    }

    "Unknown OS".to_string()
}

/// Detect project type by looking for common manifest files in the
/// provided working directory (or current dir when None).
/// Returns e.g. "rust", "node", "python", "make", or None.
pub fn detect_project_type(cwd: Option<&str>) -> Option<String> {
    let dir = cwd.unwrap_or(".");
    let p = Path::new(dir);

    if p.join("Cargo.toml").exists() {
        return Some("rust".to_string());
    }
    if p.join("package.json").exists() {
        return Some("node".to_string());
    }
    if p.join("pyproject.toml").exists() || p.join("setup.py").exists() || p.join("Pipfile").exists() {
        return Some("python".to_string());
    }
    if p.join("requirements.txt").exists() {
        return Some("python".to_string());
    }
    if p.join("Makefile").exists() {
        return Some("make".to_string());
    }
    if p.join("Gemfile").exists() {
        return Some("ruby".to_string());
    }

    None
}
