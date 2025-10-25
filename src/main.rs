use arboard::Clipboard;
use base64::{Engine as _, engine::general_purpose};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
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
    error_message: Option<String>,
    autocomplete_index: Option<usize>,
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
            error_message: None,
            autocomplete_index: None,
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

    fn get_available_commands() -> Vec<&'static str> {
        vec!["/base64-decode", "/base64-encode", "/copy"]
    }

    fn get_filtered_commands(&self) -> Vec<&'static str> {
        let input_text = self.input.to_string();
        if !input_text.starts_with('/') {
            return vec![];
        }

        Self::get_available_commands()
            .into_iter()
            .filter(|cmd| cmd.starts_with(&input_text))
            .collect()
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(mouse) => self.handle_mouse_event(mouse),
            Event::Paste(text) => {
                let text_len = text.chars().count();
                self.input.insert(self.cursor_pos, &text);
                self.cursor_pos += text_len;
                self.autocomplete_index = None;
            }
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
                    let text_len = text.chars().count();
                    self.input.insert(self.cursor_pos, &text);
                    self.cursor_pos += text_len;
                }
            }
            KeyCode::Tab => {
                let filtered = self.get_filtered_commands();
                if !filtered.is_empty() {
                    if let Some(index) = self.autocomplete_index {
                        self.autocomplete_index = Some((index + 1) % filtered.len());
                    } else {
                        self.autocomplete_index = Some(0);
                    }
                }
            }
            KeyCode::Char(c) => {
                self.input.insert_char(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.autocomplete_index = None;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos..self.cursor_pos + 1);
                    self.autocomplete_index = None;
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len_chars() {
                    self.input.remove(self.cursor_pos..self.cursor_pos + 1);
                    self.autocomplete_index = None;
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
                // Check if autocomplete is active
                let filtered = self.get_filtered_commands();
                if let Some(index) = self.autocomplete_index {
                    if let Some(command) = filtered.get(index) {
                        self.input = Rope::from(*command);
                        self.cursor_pos = self.input.len_chars();
                        self.autocomplete_index = None;
                        return;
                    }
                }

                if self.input.len_chars() > 0 {
                    let input_text = self.input.to_string();

                    // Check if it's a slash command
                    if input_text.starts_with('/') {
                        self.handle_command(&input_text);
                    } else {
                        self.buffer = input_text;
                    }

                    self.input = Rope::new();
                    self.cursor_pos = 0;
                    self.autocomplete_index = None;
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

    fn handle_command(&mut self, command: &str) {
        // Clear any previous error
        self.error_message = None;

        match command.trim() {
            "/base64-decode" => {
                if self.buffer.is_empty() {
                    self.error_message = Some("Error: Buffer is empty".to_string());
                    return;
                }

                match general_purpose::STANDARD.decode(self.buffer.trim()) {
                    Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                        Ok(decoded_string) => {
                            self.buffer = decoded_string;
                            self.scroll_pos = 0;
                        }
                        Err(_) => {
                            self.error_message =
                                Some("Error: Decoded data is not valid UTF-8".to_string());
                        }
                    },
                    Err(_) => {
                        self.error_message = Some("Error: Invalid base64 input".to_string());
                    }
                }
            }
            "/base64-encode" => {
                if self.buffer.is_empty() {
                    self.error_message = Some("Error: Buffer is empty".to_string());
                    return;
                }

                let encoded = general_purpose::STANDARD.encode(self.buffer.as_bytes());
                self.buffer = encoded;
                self.scroll_pos = 0;
            }
            "/copy" => {
                if self.buffer.is_empty() {
                    self.error_message = Some("Error: Buffer is empty".to_string());
                    return;
                }

                match self.clipboard.set_text(&self.buffer) {
                    Ok(_) => {
                        // Successfully copied, no error message needed
                    }
                    Err(_) => {
                        self.error_message = Some("Error: Failed to copy to clipboard".to_string());
                    }
                }
            }
            _ => {
                self.error_message = Some(format!("Error: Unknown command '{}'", command));
            }
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        // Split the main area into buffer, input, and error sections
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);

        // Render buffer with POMP title in the top chunk
        let title = Line::from(" POMP ".bold());
        let buffer_block = Block::bordered()
            .title(title.centered())
            .border_set(border::EMPTY);

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

        // Render input with top and bottom borders that reach the edges
        let input_block = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_set(border::PLAIN);

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
            .render(chunks[1], buf);

        // Render autocomplete popup if input starts with '/'
        let filtered_commands = self.get_filtered_commands();
        if !filtered_commands.is_empty() {
            let popup_height = (filtered_commands.len() as u16 + 2).min(10);
            let popup_width = 30;

            // Position popup above the input box
            let popup_x = chunks[1].x;
            let popup_y = chunks[1].y.saturating_sub(popup_height);

            let popup_area = Rect {
                x: popup_x,
                y: popup_y,
                width: popup_width.min(chunks[1].width),
                height: popup_height,
            };

            let items: Vec<ListItem> = filtered_commands
                .iter()
                .enumerate()
                .map(|(i, cmd)| {
                    let item = ListItem::new(*cmd);
                    if Some(i) == self.autocomplete_index {
                        item.style(Style::default().bg(Color::White).fg(Color::Black))
                    } else {
                        item
                    }
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::bordered()
                        .title("Commands")
                        .border_set(border::PLAIN),
                )
                .style(Style::default().bg(Color::Black));

            list.render(popup_area, buf);
        }

        // Render error message area
        if let Some(error) = &self.error_message {
            Paragraph::new(error.as_str())
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                .render(chunks[2], buf);
        }
    }
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::event::EnableMouseCapture,
        crossterm::event::EnableBracketedPaste
    )?;

    let mut app = App::default();
    let result = app.run(&mut terminal);

    crossterm::execute!(
        std::io::stdout(),
        crossterm::event::DisableMouseCapture,
        crossterm::event::DisableBracketedPaste
    )?;
    ratatui::restore();
    result
}
