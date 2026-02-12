use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::BorderType;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::Stdout;

use crate::App;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn init() -> anyhow::Result<Self> {
        let mut stdout = std::io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Tui { terminal })
    }

    pub fn exit(&mut self) -> anyhow::Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, Show)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn draw(&mut self, app_state: &App) -> anyhow::Result<()> {
        let duck_title = if app_state.has_git_context {
            " The Duck (Context Aware) 🦆 "
        } else {
            " The Duck 🦆 "
        };

        self.terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(20), Constraint::Min(3)])
                .split(size);

            let border_style = Style::default().fg(Color::Cyan);
            let text_style = Style::default().fg(Color::White).bg(Color::Reset);

            let error_block = Paragraph::new(app_state.error_log.as_ref())
                .block(
                    Block::default()
                        .title(" Error Context ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(border_style),
                )
                .style(text_style);

            f.render_widget(error_block, chunks[0]);

            let duck_block = Paragraph::new(app_state.duck_response.as_ref())
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title(duck_title)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(border_style),
                )
                .style(text_style);

            f.render_widget(duck_block, chunks[1]);
        })?;

        Ok(())
    }
}
