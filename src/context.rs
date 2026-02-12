use std::process::Command;

/// get_git_diff: returns recent git diff if available (stubbed).
pub fn get_git_diff() -> Option<String> {
    // Try to run `git diff HEAD` in the current repo; if it fails, return None.
    match Command::new("git").arg("diff").arg("HEAD").output() {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).into_owned();
            Some(s)
        }
        _ => None,
    }
}
