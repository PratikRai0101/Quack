/// Minimal TUI stubs. Real implementation uses ratatui + crossterm.
pub struct Tui;

impl Tui {
    pub fn init() -> anyhow::Result<Self> {
        Ok(Tui)
    }
    pub fn teardown(&mut self) {}
    pub fn draw(&mut self, _error_ctx: &str, _duck_resp: &str) {}
}
