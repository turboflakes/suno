use crate::layout::centered_area;
use qrcodegen::{QrCode, QrCodeEcc};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Clear, Paragraph, StatefulWidget, Widget},
};
use ratatui_image::{protocol::StatefulProtocol, FilterType, Resize, StatefulImage};

/// Renders QrCode data as a QR code image within the render area.
pub struct QrCodeWidget<'a> {
    data: &'a [u8],
    block: Option<Block<'a>>,
    style: Style,
    ecc: QrCodeEcc,
}

impl<'a> QrCodeWidget<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            block: None,
            style: Style::default(),
            ecc: QrCodeEcc::Low,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn set_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn ecc(mut self, ecc: QrCodeEcc) -> Self {
        self.ecc = ecc;
        self
    }
}

impl Widget for QrCodeWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        let qr = match QrCode::encode_binary(self.data, self.ecc) {
            Ok(q) => q,
            Err(_) => return,
        };

        // qrcodegen's get_module() returns false (white) for any out-of-bounds
        // coordinate, so iterating -QUIET..size+QUIET gives the quiet zone for free.
        const QUIET: i32 = 4;
        let size = qr.size(); // i32, symbol modules only (no quiet zone)
        let padded = size + QUIET * 2; // total columns/rows to render

        // The QR occupies `padded` columns and ceil(padded/2) rows (half-blocks
        // pack two module-rows per terminal row). Center that within `inner`.
        let width = padded as u16;
        let height = ((padded + 1) / 2) as u16;
        let inner = centered_area(inner, width, height);

        for ty in 0..(padded + 1) / 2 {
            let y = inner.y + ty as u16;
            if y >= inner.y + inner.height {
                break;
            }

            let top_row = ty * 2 - QUIET;
            let bottom_row = ty * 2 - QUIET + 1;

            for tx in 0..padded {
                let x = inner.x + tx as u16;
                if x >= inner.x + inner.width {
                    break;
                }

                let qr_x = tx - QUIET;
                let top_dark = qr.get_module(qr_x, top_row);
                let bottom_dark = qr.get_module(qr_x, bottom_row);

                let ch = match (top_dark, bottom_dark) {
                    (true, true) => "█",
                    (true, false) => "▀",
                    (false, true) => "▄",
                    (false, false) => " ",
                };
                buf[(x, y)].set_symbol(ch).set_style(self.style);
            }
        }
    }
}

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
            .size_for(Resize::Fit(Some(self.filter)), inner);
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
