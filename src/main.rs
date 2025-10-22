use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget},
};
use std::io::Result;

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    input: String,
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
        if matches!(event::read()?, Event::Key(_)) {
            self.exit = true;
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

        Paragraph::new("hi there")
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
