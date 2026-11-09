use crate::layout::centered_area;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};
use ratatui_image::{protocol::StatefulProtocol, FilterType, Resize, StatefulImage};

/// Renders a captured camera frame (a ratatui-image `StatefulProtocol`) fitted
/// and centered within the render area, over a solid background so the
/// letterbox margins aren't the terminal's default background.
pub struct ScannerWidget<'a> {
    protocol: &'a mut StatefulProtocol,
    block: Option<Block<'a>>,
    title: &'a str,
    title_style: Style,
    helper: &'a str,
    helper_style: Style,
    style: Style,
    filter: FilterType,
}

impl<'a> ScannerWidget<'a> {
    pub fn new(protocol: &'a mut StatefulProtocol) -> Self {
        Self {
            protocol,
            block: None,
            title: "",
            title_style: Style::default(),
            helper: "",
            helper_style: Style::default(),
            style: Style::default(),
            filter: FilterType::Nearest,
        }
    }

    // Block to use for the QR code scanner.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Title text to display above the QR code scanner.
    pub fn set_title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Title text style to display above the QR code scanner.
    pub fn set_title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Helper text to display below the QR code scanner .
    pub fn set_helper(mut self, helper: &'a str) -> Self {
        self.helper = helper;
        self
    }

    /// Helper text style to display below the QR code scanner.
    pub fn set_helper_style(mut self, style: Style) -> Self {
        self.helper_style = style;
        self
    }

    /// Background painted across the area (shows as the letterbox margins).
    pub fn set_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Resize filter used when fitting the image (default `Nearest`, fastest).
    pub fn set_filter(mut self, filter: FilterType) -> Self {
        self.filter = filter;
        self
    }
}

impl Widget for ScannerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        buf.set_style(inner, self.style);

        // Reserve a row below the image for the helper text.
        let image_area = if self.helper.is_empty() {
            inner
        } else {
            Rect {
                height: inner.height.saturating_sub(1),
                ..inner
            }
        };

        let fitted = self
            .protocol
            .size_for(Resize::Fit(Some(self.filter)), image_area.as_size());
        let centered = centered_area(image_area, fitted.width, fitted.height);

        // Set the title if it's not empty. Just above the centered image.
        if !self.title.is_empty() {
            let title_area = Rect {
                x: centered.x,
                y: centered.y.saturating_sub(1),
                width: centered.width,
                height: 1,
            };

            Paragraph::new(self.title)
                .style(self.title_style)
                .render(title_area, buf);
        }

        // Set the helper text just below the centered image.
        if !self.helper.is_empty() {
            let helper_area = Rect {
                x: centered.x,
                y: centered.y + centered.height,
                width: centered.width,
                height: 1,
            };
            if helper_area.bottom() <= inner.bottom() {
                Paragraph::new(self.helper)
                    .alignment(Alignment::Right)
                    .style(self.helper_style)
                    .render(helper_area, buf);
            }
        }

        Clear.render(centered, buf);
        StatefulImage::new()
            .resize(Resize::Fit(Some(self.filter)))
            .render(centered, buf, self.protocol);
    }
}
