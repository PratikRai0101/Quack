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
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Min(3),
                    Constraint::Length(1),
                ])
                .split(size);

            // Stealth aesthetic: muted gray borders, transparent backgrounds
            let border_style = Style::default().fg(Color::Indexed(240));
            let text_style = Style::default().bg(Color::Reset);

            // Title style: bold, default terminal color
            let title_style = Style::default().add_modifier(Modifier::BOLD);

            let error_block = Paragraph::new(app_state.error_log.as_ref())
                .block(
                    Block::default()
                        .title(Spans::from(Span::styled(" ERROR CONTEXT ", title_style)))
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

            // Start analysis with a persistent assistant prompt
            spans.push(Spans::from(Span::styled("🦆 Quack >", title_style)));

            for line in app_state.duck_response.lines() {
                let trimmed = line.trim_end();

                if trimmed.starts_with("```") {
                    in_code = !in_code;
                    // add the fence line as dim text
                    spans.push(Spans::from(Span::styled(
                        trimmed.to_string(),
                        Style::default().add_modifier(Modifier::DIM),
                    )));
                    continue;
                }

                // Detect headers to enter/exit sections (case-insensitive)
                if trimmed.to_lowercase().contains("the glitch") {
                    in_glitch = true;
                    spans.push(Spans::from(Span::styled(
                        trimmed.to_uppercase(),
                        title_style,
                    )));
                    continue;
                }
                if trimmed.to_lowercase().contains("the solution")
                    || trimmed.to_lowercase().contains("pro-tip")
                {
                    in_glitch = false;
                    spans.push(Spans::from(Span::styled(
                        trimmed.to_uppercase(),
                        title_style,
                    )));
                    continue;
                }

                if in_code {
                    // code lines: style entire line green with a darker background to simulate a block
                    spans.push(Spans::from(Span::styled(
                        trimmed.to_string(),
                        Style::default().fg(Color::Green).bg(Color::Indexed(234)),
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

                // Default: plain text; dim metadata like OS or timestamps or contextual tip
                if trimmed.starts_with("OS:")
                    || trimmed.starts_with("when:")
                    || trimmed.starts_with('#')
                {
                    spans.push(Spans::from(Span::styled(
                        trimmed.to_string(),
                        Style::default()
                            .fg(Color::Indexed(240))
                            .add_modifier(Modifier::DIM),
                    )));
                } else if trimmed.to_lowercase().starts_with("pro-tip")
                    || trimmed.to_lowercase().starts_with("contextual tip")
                {
                    spans.push(Spans::from(Span::styled(
                        trimmed.to_string(),
                        Style::default()
                            .fg(Color::Indexed(240))
                            .add_modifier(Modifier::DIM),
                    )));
                } else {
                    spans.push(Spans::from(Span::raw(trimmed.to_string())));
                }
            }

            let duck_block = Paragraph::new(spans)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title(Spans::from(Span::styled(
                            duck_title.to_uppercase(),
                            title_style,
                        )))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(border_style),
                )
                .style(text_style);

            // Render duck block in the middle chunk (no left bar — use full width)
            f.render_widget(duck_block, chunks[1]);

            // Footer: interactive one-liner
            let footer = Paragraph::new(Spans::from(vec![
                Span::styled("[q]", Style::default().fg(Color::Cyan)),
                Span::styled(" Quit  ", Style::default().add_modifier(Modifier::DIM)),
                Span::styled("[y]", Style::default().fg(Color::Cyan)),
                Span::styled(" Copy Fix  ", Style::default().add_modifier(Modifier::DIM)),
                Span::styled("[r]", Style::default().fg(Color::Cyan)),
                Span::styled(" Run Again", Style::default().add_modifier(Modifier::DIM)),
            ]))
            .style(Style::default())
            .block(Block::default());

            f.render_widget(footer, chunks[2]);
        })?;

        Ok(())
    }
}
