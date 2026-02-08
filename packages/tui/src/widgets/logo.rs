use crate::theme::THEME;
use indoc::indoc;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Padding, Paragraph, Widget},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Logo {
    size: Size,
}

/// The size of the logo
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Size {
    /// The small size of the logo (1x5 characters)
    ///
    /// ```text
    /// suno v0.1.0
    /// ```
    #[default]
    Inline,
    /// The medium version of the logo (4x34 characters)
    ///
    /// ```text
    /// █▀▀▀  █  █  █▀▀▄  █▀▀█
    /// ▀▀▀█  █  █  █  █  █  █
    /// ▀▀▀▀  ▀▀▀   ▀  ▀  ▀▀▀▀
    /// ```
    Medium,
}

impl Logo {
    pub const fn new(size: Size) -> Self {
        Self { size }
    }

    #[must_use]
    pub const fn size(self, size: Size) -> Self {
        let _ = self;
        Self { size }
    }

    pub const fn inline() -> Self {
        Self::new(Size::Inline)
    }

    pub const fn medium() -> Self {
        Self::new(Size::Medium)
    }
}

impl Widget for &Logo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logo = self.size.as_str();
        let block = Block::default()
            .style(THEME.block.menu_bottom)
            .padding(Padding::new(2, 0, 0, 1));
        Paragraph::new(logo).block(block).render(area, buf);
    }
}

impl Size {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Inline => Self::inline(),
            Self::Medium => Self::medium(),
        }
    }

    const fn inline() -> &'static str {
        concat!("suno v", env!("CARGO_PKG_VERSION"))
    }

    const fn medium() -> &'static str {
        indoc! {"
           █▀▀▀  █  █  █▀▀▄  █▀▀█
           ▀▀▀█  █  █  █  █  █  █
           ▀▀▀▀  ▀▀▀   ▀  ▀  ▀▀▀▀
        "}
    }
}
