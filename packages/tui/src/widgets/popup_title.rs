use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use suno_config::CONFIG;

pub struct PopupTitleWidget<'a> {
    title: &'a str,
    label: &'a str,
    block: Option<Block<'a>>,
}

impl<'a> PopupTitleWidget<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            label: "",
            block: None,
        }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl Widget for PopupTitleWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();
        let mut content = Vec::new();

        if !self.title.is_empty() {
            content.push(Span::styled(self.title, theme.paragraph.header(true)));
        }

        if !self.label.is_empty() {
            content.push(Span::styled(
                format!(" ({})", self.label),
                theme.paragraph.label(true),
            ));
        }

        let title = Paragraph::new(Line::from(content).alignment(Alignment::Right))
            .wrap(Wrap { trim: false });

        match self.block {
            Some(block) => title.block(block).render(area, buf),
            None => title.render(area, buf),
        }
    }
}
