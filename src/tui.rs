use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Span, Spans};
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

            // Stealth aesthetic: muted gray borders, transparent backgrounds
            let border_style = Style::default().fg(Color::Indexed(240));
            let text_style = Style::default().bg(Color::Reset);

            // Title style: bold, default terminal color
            let title_style = Style::default().add_modifier(Modifier::BOLD);

            let error_block = Paragraph::new(app_state.error_log.as_ref())
                .block(
                    Block::default()
                        .title(" Error Context ")
                        .title_style(title_style)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(border_style),
                )
                .style(text_style);

            // Apply a small horizontal padding (1 char) by shrinking the rect
            let error_area = if chunks[0].width > 2 {
                Rect {
                    x: chunks[0].x + 1,
                    y: chunks[0].y,
                    width: chunks[0].width - 2,
                    height: chunks[0].height,
                }
            } else {
                chunks[0]
            };

            f.render_widget(error_block, error_area);

            // Semantic highlighting parser:
            // - Detect fenced code blocks (```), style code as green
            // - Detect a 'The Glitch' section and highlight flag tokens (start with '-') in red
            let mut in_code = false;
            let mut in_glitch = false;
            let mut spans: Vec<Spans> = Vec::new();

            for line in app_state.duck_response.lines() {
                let trimmed = line.trim_end();

                if trimmed.starts_with("```") {
                    in_code = !in_code;
                    // add the fence line as dim text
                    spans.push(Spans::from(Span::raw(trimmed)));
                    continue;
                }

                // Detect headers to enter/exit sections
                if trimmed.contains("The Glitch") {
                    in_glitch = true;
                    spans.push(Spans::from(Span::styled(trimmed, title_style)));
                    continue;
                }
                if trimmed.contains("The Solution") || trimmed.contains("Pro-Tip") {
                    in_glitch = false;
                    spans.push(Spans::from(Span::styled(trimmed, title_style)));
                    continue;
                }

                if in_code {
                    // code lines: style entire line green
                    spans.push(Spans::from(Span::styled(
                        trimmed.to_string(),
                        Style::default().fg(Color::Green),
                    )));
                    continue;
                }

                if in_glitch {
                    // highlight flag-like tokens in red
                    let mut line_spans: Vec<Span> = Vec::new();
                    for token in trimmed.split_whitespace() {
                        if token.starts_with('-') {
                            line_spans.push(Span::styled(
                                format!("{} ", token),
                                Style::default().fg(Color::Red),
                            ));
                        } else {
                            line_spans.push(Span::raw(format!("{} ", token)));
                        }
                    }
                    spans.push(Spans::from(line_spans));
                    continue;
                }

                // Default: plain text
                spans.push(Spans::from(Span::raw(trimmed.to_string())));
            }

            let duck_block = Paragraph::new(spans)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title(duck_title)
                        .title_style(title_style)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(border_style),
                )
                .style(text_style);

            let duck_area = if chunks[1].width > 2 {
                Rect {
                    x: chunks[1].x + 1,
                    y: chunks[1].y,
                    width: chunks[1].width - 2,
                    height: chunks[1].height,
                }
            } else {
                chunks[1]
            };

            f.render_widget(duck_block, duck_area);
        })?;

        Ok(())
    }
}
