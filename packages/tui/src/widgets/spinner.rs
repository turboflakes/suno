use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Padding, Paragraph, Widget},
};
use std::time::Instant;
use suno_config::CONFIG;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spinner {
    frames: Vec<&'static str>,
    start_time: Instant,
    counter: usize,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⢸", "⣸", "⣠", "⣄", "⣇", "⠇", "⠏"],
            start_time: Instant::now(),
            counter: 0,
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

    pub fn progress(&self) -> String {
        let full = "⣿".repeat(self.counter);
        format!("{}{}", full, self.frame())
    }

    pub fn increment(&mut self) {
        self.counter += 1;
    }
}

impl Widget for &Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();
        let spinner = self.progress();
        let block = Block::new()
            .style(theme.input.base_active)
            .padding(Padding::new(0, 2, 1, 1));
        Paragraph::new(spinner)
            .block(block)
            .style(theme.input.suffix_active)
            .render(area, buf);
    }
}
