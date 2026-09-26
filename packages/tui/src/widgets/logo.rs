use indoc::indoc;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use suno_theme::Theme;

#[derive(Debug, Default, Clone)]
pub struct Logo {
    size: Size,
    theme: Theme,
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
    /// The large version of the logo with shadow (21x36 characters)
    Original,
}

impl Logo {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            theme: Theme::default(),
        }
    }

    #[must_use]
    pub fn size(self, size: Size) -> Self {
        Self { size, ..self }
    }

    #[must_use]
    pub fn theme(self, theme: Theme) -> Self {
        Self { theme, ..self }
    }

    pub fn inline() -> Self {
        Self::new(Size::Inline)
    }

    pub fn medium() -> Self {
        Self::new(Size::Medium)
    }

    pub fn large() -> Self {
        Self::new(Size::Large)
    }

    pub fn original() -> Self {
        Self::new(Size::Original)
    }

    pub fn render_original(&self, area: Rect, buf: &mut Buffer) {
        let theme = self.theme;
        let mut lines = vec![];
        lines.push(Line::from(vec![
            Span::styled("            ", theme.logo.base),
            Span::styled("▀█", theme.logo.base),
            Span::styled("▀", theme.logo.with_shadow),
            Span::styled("▘", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("       ", theme.logo.base),
            Span::styled("▄▄████████████████████", theme.logo.base),
            Span::styled("▄", theme.logo.with_shadow),
            Span::styled("▄", theme.logo.base),
            Span::styled("▖", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    ", theme.logo.base),
            Span::styled("▄████████████████████████████", theme.logo.base),
            Span::styled("▄", theme.logo.with_shadow),
            Span::styled("▖", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   ", theme.logo.base),
            Span::styled("████████████████████████████████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", theme.logo.base),
            Span::styled("█████", theme.logo.base),
            Span::styled("▀", theme.logo.with_shadow),
            Span::styled("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", theme.logo.base),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████████████████████████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", theme.logo.base),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████", theme.logo.base),
            Span::styled("▛", theme.logo.with_shadow),
            Span::styled("▀██████████████████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", theme.logo.base),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("██████", theme.logo.base),
            Span::styled("▙", theme.logo.with_shadow),
            Span::styled("▄", theme.logo.base),
            Span::styled("▛", theme.logo.with_shadow),
            Span::styled("▀██████████████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" ", theme.logo.base),
            Span::styled("▄████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████", theme.logo.base),
            Span::styled("▛", theme.logo.with_shadow),
            Span::styled("▀", theme.logo.base),
            Span::styled("▙", theme.logo.with_shadow),
            Span::styled("▄██████", theme.logo.base),
            Span::styled("▛", theme.logo.with_shadow),
            Span::styled("▀▀▀▀▀████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████▙", theme.logo.base),
            Span::styled("▖", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" ", theme.logo.base),
            Span::styled("█████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████████████████████████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("█████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" ", theme.logo.base),
            Span::styled("▀█████", theme.logo.base),
            Span::styled("▄", theme.logo.with_shadow),
            Span::styled("▄▄▄▄▄▄▄▄▄▄", theme.logo.base),
            Span::styled("▖", theme.logo.only_shadow),
            Span::styled("█", theme.logo.base_dark),
            Span::styled("▄▄▄▄▄▄▄▄▄▄▄█████▛", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", theme.logo.base),
            Span::styled("██████████████", theme.logo.base),
            Span::styled("▛", theme.logo.with_shadow),
            Span::styled("▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("█", theme.logo.base_dark),
            Span::styled("▀▀██████████████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", theme.logo.base),
            Span::styled("██████████████████████████████████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", theme.logo.base),
            Span::styled("█████", theme.logo.base),
            Span::styled("▛", theme.logo.with_shadow),
            Span::styled("▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀█████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ", theme.logo.base),
            Span::styled("▀████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("▀▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀█", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("████▛", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   ", theme.logo.base),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   ", theme.logo.base),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
            Span::styled("▀▀▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀▀▀▀▀▀▀▀▀▀▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("▀▀▀", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
            Span::styled("████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   ", theme.logo.base),
            Span::styled("████████████████████████████████", theme.logo.base),
            Span::styled("▌", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![
            Span::styled("   ", theme.logo.base),
            Span::styled("▀██████████████████████████████▛", theme.logo.base),
            Span::styled("▘", theme.logo.only_shadow),
        ]));
        lines.push(Line::from(vec![Span::styled(
            format!("    suno v{}", env!("CARGO_PKG_VERSION")),
            theme.logo.base,
        )]));

        Paragraph::new(lines)
            .alignment(Alignment::Left)
            .style(theme.logo.base)
            .render(area, buf);
    }
}

impl Size {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Inline => Self::inline(),
            Self::Medium => Self::medium(),
            Self::Large => Self::large(),
            _ => "",
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

impl Widget for &Logo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.size == Size::Original {
            self.render_original(area, buf);
            return;
        }

        let logo = self.size.as_str();
        Paragraph::new(logo).render(area, buf);
    }
}
