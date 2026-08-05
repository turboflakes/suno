use crate::layout::centered_area;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};
use ratatui_image::{protocol::StatefulProtocol, FilterType, Resize, StatefulImage};

/// Renders a captured camera frame (a ratatui-image `StatefulProtocol`) fitted
/// and centered within the render area, over a solid background so the
/// letterbox margins aren't the terminal's default background.
pub struct QrScannerWidget<'a> {
    protocol: &'a mut StatefulProtocol,
    block: Option<Block<'a>>,
    title: &'a str,
    title_style: Style,
    style: Style,
    filter: FilterType,
}

impl<'a> QrScannerWidget<'a> {
    pub fn new(protocol: &'a mut StatefulProtocol) -> Self {
        Self {
            protocol,
            block: None,
            title: "",
            title_style: Style::default(),
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

impl Widget for QrScannerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        buf.set_style(inner, self.style);

        let fitted = self
            .protocol
            .size_for(Resize::Fit(Some(self.filter)), inner.as_size());
        let centered = centered_area(inner, fitted.width, fitted.height);

        let [title_area, scanner_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)])
            .areas(centered);

        Clear.render(scanner_area, buf);
        Paragraph::new(self.title)
            .style(self.title_style)
            .render(title_area, buf);

        StatefulImage::new()
            .resize(Resize::Fit(Some(self.filter)))
            .render(scanner_area, buf, self.protocol);
    }
}
