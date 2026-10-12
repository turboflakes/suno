use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Padding, Paragraph, Widget},
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use suno_theme::Theme;

const FRAMES: [&str; 11] = ["⠋", "⠙", "⠹", "⠸", "⢸", "⣸", "⣠", "⣄", "⣇", "⠇", "⠏"];
const FRAME_MILLIS: u64 = 250;

/// Current spinner frame based on the wall clock, for stateless widgets (e.g. table rows)
/// that have no [`Spinner`] instance of their own but should animate in sync.
pub fn current_frame() -> &'static str {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default();
    FRAMES[(millis / FRAME_MILLIS) as usize % FRAMES.len()]
}

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
            frames: FRAMES.to_vec(),
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
        let frame_index = (elapsed / FRAME_MILLIS) as usize % self.frames.len();
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

pub fn render_spinner(theme: Theme, spinner: &Spinner, area: Rect, buf: &mut Buffer) {
    let status = spinner.status();
    let block = Block::new()
        .style(theme.input.base)
        .padding(Padding::new(0, 2, 1, 1));
    Paragraph::new(status)
        .block(block)
        .style(theme.input.suffix)
        .render(area, buf);
}
