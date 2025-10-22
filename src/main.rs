use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    prelude::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
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
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);
        Paragraph::new("hi there")
            .centered()
            .block(block)
            .render(area, buf)
    }
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::default();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
