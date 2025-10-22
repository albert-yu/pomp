use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget},
};
use ropey::Rope;
use std::io::Result;

#[derive(Debug)]
pub struct App {
    exit: bool,
    input: Rope,
    cursor_pos: usize,
    buffer: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            input: Rope::new(),
            cursor_pos: 0,
            buffer: String::new(),
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('d')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.exit = true;
                }
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.cursor_pos = 0;
                }
                KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.cursor_pos = self.input.len_chars();
                }
                KeyCode::Char(c) => {
                    self.input.insert_char(self.cursor_pos, c);
                    self.cursor_pos += 1;
                }
                KeyCode::Backspace => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        self.input.remove(self.cursor_pos..self.cursor_pos + 1);
                    }
                }
                KeyCode::Delete => {
                    if self.cursor_pos < self.input.len_chars() {
                        self.input.remove(self.cursor_pos..self.cursor_pos + 1);
                    }
                }
                KeyCode::Left => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.cursor_pos < self.input.len_chars() {
                        self.cursor_pos += 1;
                    }
                }
                KeyCode::Home => {
                    self.cursor_pos = 0;
                }
                KeyCode::End => {
                    self.cursor_pos = self.input.len_chars();
                }
                KeyCode::Enter => {
                    if self.input.len_chars() > 0 {
                        self.buffer = self.input.to_string();
                        self.input = Rope::new();
                        self.cursor_pos = 0;
                    }
                }
                KeyCode::Esc => {
                    self.exit = true;
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" POMP ".bold());
        let container = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);
        let inner_area = container.inner(area);
        container.render(area, buf);

        let chunks =
            Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(inner_area);

        let input_title = Line::from("Input");

        let input_block = Block::bordered()
            .title(input_title.left_aligned())
            .border_set(border::PLAIN)
            .border_type(BorderType::Rounded);

        let input_len = self.input.len_chars();
        let text_with_cursor = if self.cursor_pos >= input_len {
            format!("{}█", self.input)
        } else {
            let before = self.input.slice(..self.cursor_pos).to_string();
            let after = self.input.slice(self.cursor_pos + 1..).to_string();
            format!("{}█{}", before, after)
        };

        Paragraph::new(text_with_cursor)
            .block(input_block)
            .render(chunks[1], buf)
    }
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::default();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
