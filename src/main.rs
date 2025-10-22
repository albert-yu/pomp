use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
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

pub struct App {
    exit: bool,
    input: Rope,
    cursor_pos: usize,
    buffer: String,
    scroll_pos: usize,
    clipboard: Clipboard,
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            input: Rope::new(),
            cursor_pos: 0,
            buffer: String::new(),
            scroll_pos: 0,
            clipboard: Clipboard::new().unwrap(),
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
        match event::read()? {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(mouse) => self.handle_mouse_event(mouse),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
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
            KeyCode::Char('v')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::SUPER) =>
            {
                if let Ok(text) = self.clipboard.get_text() {
                    for ch in text.chars() {
                        self.input.insert_char(self.cursor_pos, ch);
                        self.cursor_pos += 1;
                    }
                }
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
            KeyCode::PageUp => {
                self.scroll_pos = self.scroll_pos.saturating_sub(10);
            }
            KeyCode::PageDown => {
                let buffer_lines = self.buffer.lines().count();
                if buffer_lines > 0 {
                    self.scroll_pos = (self.scroll_pos + 10).min(buffer_lines.saturating_sub(1));
                }
            }
            KeyCode::Esc => {
                self.exit = true;
            }
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.scroll_pos = self.scroll_pos.saturating_sub(3);
            }
            MouseEventKind::ScrollDown => {
                let buffer_lines = self.buffer.lines().count();
                if buffer_lines > 0 {
                    self.scroll_pos = (self.scroll_pos + 3).min(buffer_lines.saturating_sub(1));
                }
            }
            _ => {}
        }
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

        // Render buffer in the top chunk
        let buffer_block = Block::bordered().border_set(border::EMPTY);

        let buffer_inner = buffer_block.inner(chunks[0]);
        let visible_height = buffer_inner.height as usize;

        // Split buffer into lines and calculate visible portion
        let buffer_lines: Vec<&str> = self.buffer.lines().collect();
        let total_lines = buffer_lines.len();
        let start_line = self.scroll_pos.min(total_lines.saturating_sub(1));
        let end_line = (start_line + visible_height).min(total_lines);

        let visible_text = if buffer_lines.is_empty() {
            String::new()
        } else {
            buffer_lines[start_line..end_line].join("\n")
        };

        Paragraph::new(visible_text)
            .block(buffer_block)
            .render(chunks[0], buf);

        let input_block = Block::bordered()
            .border_set(border::PLAIN)
            .border_type(BorderType::Rounded);

        let input_len = self.input.len_chars();
        let text_with_cursor = if self.cursor_pos >= input_len {
            format!("> {}█", self.input)
        } else {
            let before = self.input.slice(..self.cursor_pos).to_string();
            let after = self.input.slice(self.cursor_pos + 1..).to_string();
            format!("> {}█{}", before, after)
        };

        Paragraph::new(text_with_cursor)
            .block(input_block)
            .render(chunks[1], buf)
    }
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;

    let mut app = App::default();
    let result = app.run(&mut terminal);

    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)?;
    ratatui::restore();
    result
}
