/// Minimal TUI implementation used for v0.1 polishing.
/// It keeps a simple flag whether git context is available and
/// renders the error context and the duck response (accumulated).
pub struct Tui {
    pub has_git_context: bool,
}

impl Tui {
    /// Initialize the TUI. In a fuller implementation this would enable
    /// raw mode and set up the terminal. Here we keep it minimal but
    /// provide a teardown hook the caller must call to restore state.
    pub fn init(has_git_context: bool) -> anyhow::Result<Self> {
        Ok(Tui { has_git_context })
    }

    /// Restore terminal state. Placeholder for real teardown logic.
    pub fn teardown(&mut self) {}

    /// Draw the UI. This minimal implementation prints a compact
    /// representation to stdout so the app can be used without the full
    /// ratatui dependency wired into this simple scaffold.
    pub fn draw(&mut self, error_ctx: &str, duck_resp: &str) {
        // Compose duck title based on whether we have git context.
        let duck_title = if self.has_git_context {
            " The Duck (Context Aware) 🦆 "
        } else {
            " The Duck "
        };

        // Simple, idempotent console render: clear screen and print panes.
        // Keep it simple to avoid terminal mode dependencies in this scaffold.
        print!("\x1b[2J\x1b[H"); // clear screen, move cursor home
        println!("+-----------------------------+");
        println!(
            "| Error Context{}|",
            if self.has_git_context {
                " (Git: Detected)"
            } else {
                ""
            }
        );
        println!("+-----------------------------+");
        if error_ctx.is_empty() {
            println!("<no stderr captured>");
        } else {
            println!("{}", error_ctx);
        }
        println!("\n+-----------------------------+");
        println!("|{}|", duck_title);
        println!("+-----------------------------+");
        if duck_resp.is_empty() {
            println!("<waiting for AI response...>");
        } else {
            // Keep background transparent; print the response as-is.
            println!("{}", duck_resp);
        }
    }
}
