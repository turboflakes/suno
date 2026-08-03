use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Padding, Paragraph, Widget},
};
use std::time::Instant;
use suno_config::CONFIG;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Running,
    Completed,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spinner {
    frames: Vec<&'static str>,
    start_time: Instant,
    counter: usize,
    status: Status,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⢸", "⣸", "⣠", "⣄", "⣇", "⠇", "⠏"],
            start_time: Instant::now(),
            counter: 0,
            status: Status::Running,
        }
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn frame(&self) -> &str {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        let frame_index = (elapsed / 250) as usize % self.frames.len();
        self.frames[frame_index]
    }

    fn progress(&self) -> String {
        let mut result = "⣿".repeat(self.counter);
        result.push_str(self.frame());
        result
    }

    pub fn increment(&mut self) {
        self.status = Status::Running;
        self.counter += 1;
    }

    pub fn complete(&mut self) {
        self.status = Status::Completed;
        self.counter = 0;
    }

    pub fn error(&mut self) {
        self.status = Status::Error;
        self.counter = 0;
    }

    pub fn status(&self) -> String {
        match self.status {
            Status::Running => self.progress(),
            Status::Error => "✘".to_string(),
            Status::Completed => "✔︎".to_string(),
        }
    }
}

impl Widget for &Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();
        let spinner = self.status();
        let block = Block::new()
            .style(theme.input.base)
            .padding(Padding::new(0, 2, 1, 1));
        Paragraph::new(spinner)
            .block(block)
            .style(theme.input.suffix)
            .render(area, buf);
    }
}
