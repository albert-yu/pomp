use arboard::Clipboard;
use base64::{Engine as _, engine::general_purpose};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
};
use ropey::Rope;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Result;
use uuid::Uuid;

pub struct App {
    exit: bool,
    input: Rope,
    cursor_pos: usize,
    buffer: String,
    scroll_pos: usize,
    clipboard: Clipboard,
    error_message: Option<String>,
    info_message: Option<String>,
    autocomplete_index: Option<usize>,
    input_scroll_line: usize,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
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
            info_message: Some("Press / to show available commands".to_string()),
            autocomplete_index: None,
            input_scroll_line: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
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

    fn get_cursor_line_col(&self) -> (usize, usize) {
        let text = self.input.to_string();
        let mut line = 0;
        let mut col = 0;
        for (i, ch) in text.chars().enumerate() {
            if i >= self.cursor_pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn set_cursor_from_line_col(&mut self, target_line: usize, target_col: usize) {
        let text = self.input.to_string();
        let mut line = 0;
        let mut col = 0;
        let mut pos = 0;

        for ch in text.chars() {
            if line == target_line && col == target_col {
                break;
            }
            if line > target_line {
                break;
            }
            if ch == '\n' {
                if line == target_line {
                    break;
                }
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            pos += 1;
        }
        self.cursor_pos = pos;
    }

    fn adjust_input_scroll(&mut self) {
        let (current_line, _) = self.get_cursor_line_col();
        let max_visible_lines = 5;

        // Scroll down if cursor is below visible area
        if current_line >= self.input_scroll_line + max_visible_lines {
            self.input_scroll_line = current_line - max_visible_lines + 1;
        }

        // Scroll up if cursor is above visible area
        if current_line < self.input_scroll_line {
            self.input_scroll_line = current_line;
        }
    }

    fn get_available_commands() -> Vec<&'static str> {
        vec![
            "/base64-decode",
            "/base64-encode",
            "/copy",
            "/css-format",
            "/css-minify",
            "/cuid",
            "/exit",
            "/json-format",
            "/json-minify",
            "/sha-256",
            "/uuid",
        ]
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
                self.adjust_input_scroll();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('z')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::SUPER) =>
            {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    // Redo
                    self.redo();
                } else {
                    // Undo
                    self.undo();
                }
            }
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
                self.adjust_input_scroll();
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos..self.cursor_pos + 1);
                    self.autocomplete_index = None;
                    self.adjust_input_scroll();
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len_chars() {
                    self.input.remove(self.cursor_pos..self.cursor_pos + 1);
                    self.autocomplete_index = None;
                    self.adjust_input_scroll();
                }
            }
            KeyCode::Up => {
                let (current_line, current_col) = self.get_cursor_line_col();
                if current_line > 0 {
                    self.set_cursor_from_line_col(current_line - 1, current_col);
                    self.adjust_input_scroll();
                }
            }
            KeyCode::Down => {
                let (current_line, current_col) = self.get_cursor_line_col();
                let text = self.input.to_string();
                let total_lines = if text.is_empty() {
                    1
                } else {
                    text.lines().count().max(1)
                };
                if current_line + 1 < total_lines {
                    self.set_cursor_from_line_col(current_line + 1, current_col);
                    self.adjust_input_scroll();
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.adjust_input_scroll();
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len_chars() {
                    self.cursor_pos += 1;
                    self.adjust_input_scroll();
                }
            }
            KeyCode::Home => {
                let (current_line, _) = self.get_cursor_line_col();
                self.set_cursor_from_line_col(current_line, 0);
                self.adjust_input_scroll();
            }
            KeyCode::End => {
                let text = self.input.to_string();
                let (current_line, _) = self.get_cursor_line_col();
                let lines: Vec<&str> = text.lines().collect();
                if current_line < lines.len() {
                    let line_length = lines[current_line].chars().count();
                    self.set_cursor_from_line_col(current_line, line_length);
                } else {
                    self.cursor_pos = self.input.len_chars();
                }
                self.adjust_input_scroll();
            }
            KeyCode::Enter => {
                // Check for Shift+Enter to insert newline
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.input.insert_char(self.cursor_pos, '\n');
                    self.cursor_pos += 1;
                    self.adjust_input_scroll();
                    return;
                }

                // Check if autocomplete is active
                let filtered = self.get_filtered_commands();
                if let Some(index) = self.autocomplete_index {
                    if let Some(command) = filtered.get(index) {
                        self.input = Rope::from(*command);
                        self.cursor_pos = self.input.len_chars();
                        self.autocomplete_index = None;
                        self.input_scroll_line = 0;
                        return;
                    }
                }

                if self.input.len_chars() > 0 {
                    let input_text = self.input.to_string();
                    let input_trimmed = input_text.trim();

                    // Check if it exactly matches a valid command
                    let is_valid_command = App::get_available_commands()
                        .iter()
                        .any(|cmd| *cmd == input_trimmed);

                    if is_valid_command {
                        self.handle_command(input_trimmed);
                    } else {
                        // Save current buffer to undo stack before replacing
                        self.push_undo();
                        self.buffer = input_text;
                    }

                    self.input = Rope::new();
                    self.cursor_pos = 0;
                    self.autocomplete_index = None;
                    self.input_scroll_line = 0;
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
                // Close autocomplete popup if open, otherwise exit
                let filtered = self.get_filtered_commands();
                if !filtered.is_empty() || self.autocomplete_index.is_some() {
                    self.autocomplete_index = None;
                    self.input = Rope::new();
                    self.cursor_pos = 0;
                    self.input_scroll_line = 0;
                }
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

    fn push_undo(&mut self) {
        const MAX_STACK_SIZE: usize = 500;

        self.undo_stack.push(self.buffer.clone());

        // Keep stack size under limit
        if self.undo_stack.len() > MAX_STACK_SIZE {
            self.undo_stack.remove(0);
        }

        // Clear redo stack on new action
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(previous_buffer) = self.undo_stack.pop() {
            // Push current buffer to redo stack
            self.redo_stack.push(self.buffer.clone());

            // Restore previous buffer
            self.buffer = previous_buffer;
            self.scroll_pos = 0;
            self.info_message = Some("Undo".to_string());
        }
    }

    fn redo(&mut self) {
        if let Some(next_buffer) = self.redo_stack.pop() {
            // Push current buffer to undo stack
            self.undo_stack.push(self.buffer.clone());

            // Restore next buffer
            self.buffer = next_buffer;
            self.scroll_pos = 0;
            self.info_message = Some("Redo".to_string());
        }
    }

    fn handle_command(&mut self, command: &str) {
        // Save current buffer state before command execution
        self.push_undo();

        // Clear any previous error and info message
        self.error_message = None;
        self.info_message = None;

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
                        self.info_message = Some("Copied to clipboard".to_string());
                    }
                    Err(_) => {
                        self.error_message = Some("Error: Failed to copy to clipboard".to_string());
                    }
                }
            }
            "/json-format" => {
                if self.buffer.is_empty() {
                    self.error_message = Some("Error: Buffer is empty".to_string());
                    return;
                }

                match serde_json::from_str::<Value>(&self.buffer) {
                    Ok(json_value) => match serde_json::to_string_pretty(&json_value) {
                        Ok(formatted) => {
                            self.buffer = formatted;
                            self.scroll_pos = 0;
                        }
                        Err(_) => {
                            self.error_message = Some("Error: Failed to format JSON".to_string());
                        }
                    },
                    Err(e) => {
                        self.error_message = Some(format!("Error: Invalid JSON - {}", e));
                    }
                }
            }
            "/json-minify" => {
                if self.buffer.is_empty() {
                    self.error_message = Some("Error: Buffer is empty".to_string());
                    return;
                }

                match serde_json::from_str::<Value>(&self.buffer) {
                    Ok(json_value) => match serde_json::to_string(&json_value) {
                        Ok(minified) => {
                            self.buffer = minified;
                            self.scroll_pos = 0;
                        }
                        Err(_) => {
                            self.error_message = Some("Error: Failed to minify JSON".to_string());
                        }
                    },
                    Err(e) => {
                        self.error_message = Some(format!("Error: Invalid JSON - {}", e));
                    }
                }
            }
            "/css-format" => {
                if self.buffer.is_empty() {
                    self.error_message = Some("Error: Buffer is empty".to_string());
                    return;
                }

                let buffer_clone = self.buffer.clone();
                match StyleSheet::parse(&buffer_clone, ParserOptions::default()) {
                    Ok(stylesheet) => {
                        let printer_options = PrinterOptions {
                            minify: false,
                            ..Default::default()
                        };
                        match stylesheet.to_css(printer_options) {
                            Ok(result) => {
                                self.buffer = result.code;
                                self.scroll_pos = 0;
                            }
                            Err(_) => {
                                self.error_message =
                                    Some("Error: Failed to format CSS".to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Error: Invalid CSS - {}", e));
                    }
                }
            }
            "/css-minify" => {
                if self.buffer.is_empty() {
                    self.error_message = Some("Error: Buffer is empty".to_string());
                    return;
                }

                let buffer_clone = self.buffer.clone();
                match StyleSheet::parse(&buffer_clone, ParserOptions::default()) {
                    Ok(mut stylesheet) => {
                        if let Err(e) = stylesheet.minify(MinifyOptions::default()) {
                            self.error_message =
                                Some(format!("Error: Failed to minify CSS - {}", e));
                            return;
                        }
                        let printer_options = PrinterOptions {
                            minify: true,
                            ..Default::default()
                        };
                        match stylesheet.to_css(printer_options) {
                            Ok(result) => {
                                self.buffer = result.code;
                                self.scroll_pos = 0;
                            }
                            Err(_) => {
                                self.error_message =
                                    Some("Error: Failed to minify CSS".to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Error: Invalid CSS - {}", e));
                    }
                }
            }
            "/cuid" => {
                let new_cuid = cuid::cuid2();
                self.buffer = new_cuid;
                self.scroll_pos = 0;
            }
            "/exit" => {
                self.exit = true;
            }
            "/sha-256" => {
                if self.buffer.is_empty() {
                    self.error_message = Some("Error: Buffer is empty".to_string());
                    return;
                }

                let mut hasher = Sha256::new();
                hasher.update(self.buffer.as_bytes());
                let result = hasher.finalize();
                self.buffer = format!("{:x}", result);
                self.scroll_pos = 0;
            }
            "/uuid" => {
                let new_uuid = Uuid::new_v4();
                self.buffer = new_uuid.to_string();
                self.scroll_pos = 0;
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
        // Calculate input lines and height
        let input_text = self.input.to_string();
        let input_line_count = if input_text.is_empty() {
            1
        } else {
            input_text.lines().count().max(1)
        };
        let max_visible_lines = 5;
        let visible_input_lines = input_line_count.min(max_visible_lines);
        let input_height = visible_input_lines as u16 + 2; // +2 for borders

        // Split the main area into buffer, input, and error sections
        let chunks = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(input_height),
            Constraint::Length(1),
        ])
        .split(area);

        let title = Line::from(" pomp ".bold());
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

        // Build input text with cursor and handle multiple lines
        let all_lines: Vec<&str> = input_text.lines().collect();
        let start_line = self
            .input_scroll_line
            .min(input_line_count.saturating_sub(1));
        let end_line = (start_line + max_visible_lines).min(input_line_count);

        let visible_lines = if all_lines.is_empty() {
            vec![""]
        } else {
            all_lines[start_line..end_line].to_vec()
        };

        // Build text with cursor, adjusting for scrolled lines
        let (cursor_line, cursor_col) = self.get_cursor_line_col();

        // Add proper prefixes to each line (> for first line, spaces for continuation lines)
        let formatted_lines: Vec<String> = visible_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    format!("> {}", line)
                } else {
                    format!("  {}", line)
                }
            })
            .collect();

        let formatted_display = formatted_lines.join("\n");

        let text_with_cursor = if cursor_line >= start_line && cursor_line < end_line {
            // Cursor is in visible area
            let line_offset = cursor_line - start_line;
            let mut char_pos = 0;

            // Account for line prefixes and content
            for i in 0..line_offset {
                if i < visible_lines.len() {
                    char_pos += 2; // "> " or "  " prefix
                    char_pos += visible_lines[i].chars().count();
                    char_pos += 1; // newline
                }
            }
            char_pos += 2; // Current line prefix
            char_pos += cursor_col;

            let before: String = formatted_display.chars().take(char_pos).collect();
            let char_at_cursor = formatted_display.chars().nth(char_pos);
            let after: String = formatted_display.chars().skip(char_pos + 1).collect();

            // If cursor is on a newline, show cursor but keep the newline
            if char_at_cursor == Some('\n') {
                format!("{}█\n{}", before, after)
            } else {
                format!("{}█{}", before, after)
            }
        } else {
            // Cursor not in visible area (shouldn't happen with proper scrolling)
            format!("{}█", formatted_display)
        };

        // Check if input matches a command exactly
        let input_trimmed = input_text.trim();
        let is_valid_command = App::get_available_commands()
            .iter()
            .any(|cmd| *cmd == input_trimmed);

        let input_paragraph = if is_valid_command {
            Paragraph::new(text_with_cursor)
                .block(input_block)
                .style(Style::default().bold())
        } else {
            Paragraph::new(text_with_cursor).block(input_block)
        };

        input_paragraph.render(chunks[1], buf);

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

            // Clear the popup area to ensure opaque background
            Clear.render(popup_area, buf);

            let list = List::new(items)
                .block(
                    Block::bordered()
                        .title("Commands")
                        .border_set(border::PLAIN),
                )
                .style(Style::default().bg(Color::Black));

            list.render(popup_area, buf);
        }

        // Render error or info message area
        if let Some(error) = &self.error_message {
            Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .render(chunks[2], buf);
        } else if input_line_count > max_visible_lines {
            // Show remaining line count if input has more than 5 lines and we're at the bottom
            let remaining_above = self.input_scroll_line;
            let remaining_below =
                input_line_count.saturating_sub(self.input_scroll_line + max_visible_lines);

            let message = if remaining_above > 0 && remaining_below > 0 {
                format!(
                    "{} more line{} above, {} below",
                    remaining_above,
                    if remaining_above == 1 { "" } else { "s" },
                    remaining_below
                )
            } else if remaining_above > 0 {
                format!(
                    "{} more line{} above",
                    remaining_above,
                    if remaining_above == 1 { "" } else { "s" }
                )
            } else if remaining_below > 0 {
                format!(
                    "{} more line{} below",
                    remaining_below,
                    if remaining_below == 1 { "" } else { "s" }
                )
            } else {
                String::new()
            };

            if !message.is_empty() {
                Paragraph::new(message)
                    .style(Style::default().fg(Color::Gray))
                    .render(chunks[2], buf);
            }
        } else if let Some(info) = &self.info_message {
            Paragraph::new(info.as_str())
                .style(Style::default().fg(Color::Gray))
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
