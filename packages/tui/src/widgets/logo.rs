use indoc::indoc;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
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
    /// The large version of the logo (20x36 characters)
    ///
    /// ```text
    ///            ▀█▀
    ///       ▄▄████████████████████▄▄
    ///    ▄████████████████████████████▄
    ///   ████████████████████████████████
    ///  █████▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█████
    ///  ████ ████████████████████████ ████
    ///  ████ ████▀▀██████████████████ ████
    ///  ████ ██████▄▄▀▀██████████████ ████
    /// ▄████ ████▀▀▄▄██████▀▀▀▀▀▀████ ████▄
    /// █████ ████████████████████████ █████
    /// ▀█████▄▄▄▄▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄▄▄█████▀
    ///  ██████████████▀▀  ▀▀██████████████
    ///  ██████████████████████████████████
    ///  █████▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█████
    ///  ▀████ ▀▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀█ ████▀
    ///   ████ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀ ████
    ///   ████ ▀▀▀▀ ▀▀▀▀▀▀▀▀▀▀▀▀▀ ▀▀▀ ████
    ///   ████████████████████████████████
    ///   ▀██████████████████████████████▀
    ///    suno v0.1.0
    /// ```
    Large,
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

    pub const fn large() -> Self {
        Self::new(Size::Large)
    }
}

impl Widget for &Logo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logo = self.size.as_str();
        Paragraph::new(logo).render(area, buf);
    }
}

impl Size {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Inline => Self::inline(),
            Self::Medium => Self::medium(),
            Self::Large => Self::large(),
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

    const fn large() -> &'static str {
        concat!(
            "
            ▀█▀
       ▄▄████████████████████▄▄
    ▄████████████████████████████▄
   ████████████████████████████████
  █████▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█████
  ████ ████████████████████████ ████
  ████ ████▀▀██████████████████ ████
  ████ ██████▄▄▀▀██████████████ ████
 ▄████ ████▀▀▄▄██████▀▀▀▀▀▀████ ████▄
 █████ ████████████████████████ █████
 ▀█████▄▄▄▄▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄▄▄█████▀
  ██████████████▀▀  ▀▀██████████████
  ██████████████████████████████████
  █████▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█████
  ▀████ ▀▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀█ ████▀
   ████ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀▀ ▀ ████
   ████ ▀▀▀▀ ▀▀▀▀▀▀▀▀▀▀▀▀▀ ▀▀▀ ████
   ████████████████████████████████
   ▀██████████████████████████████▀
    suno v",
            env!("CARGO_PKG_VERSION")
        )
    }
}
