use crate::layout::centered_area;
use qrcodegen::{QrCode, QrCodeEcc};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};

const QUIET: i32 = 4;

/// Renders QrCode data as a QR code image within the render area.
pub struct QrCodeWidget<'a> {
    qr: Option<QrCode>,
    block: Option<Block<'a>>,
    style: Style,
}

impl<'a> QrCodeWidget<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let qr = QrCode::encode_binary(data, QrCodeEcc::Low).ok();
        Self {
            qr,
            block: None,
            style: Style::default(),
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn width(&self) -> u16 {
        match &self.qr {
            Some(qr) => (qr.size() + QUIET * 2) as u16,
            None => 0,
        }
    }

    pub fn height(&self) -> u16 {
        let w = self.width() as i32;
        ((w + 1) / 2) as u16
    }

    // pub fn ecc(mut self, ecc: QrCodeEcc) -> Self {
    //     self.ecc = ecc;
    //     self
    // }
}

impl Widget for QrCodeWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let qr = match self.qr {
            Some(qr) => qr,
            None => return,
        };

        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        render_qr(&qr, inner, self.style, buf);
    }
}

pub fn render_qr(qr: &QrCode, area: Rect, style: Style, buf: &mut Buffer) {
    // NOTE: qrcodegen's get_module() returns false (white) for any out-of-bounds
    // coordinate, so iterating -QUIET..size+QUIET gives the quiet zone for free.
    // The QR occupies `padded` columns and ceil(padded/2) rows (half-blocks
    // pack two module-rows per terminal row). Center that within `area`.
    let padded = qr.size() + QUIET * 2; // total columns/rows to render
    let width = padded as u16;
    let height = ((padded + 1) / 2) as u16;
    let area = centered_area(area, width, height);

    for ty in 0..(padded + 1) / 2 {
        let y = area.y + ty as u16;
        if y >= area.y + area.height {
            break;
        }

        let top_row = ty * 2 - QUIET;
        let bottom_row = ty * 2 - QUIET + 1;

        for tx in 0..padded {
            let x = area.x + tx as u16;
            if x >= area.x + area.width {
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
            buf[(x, y)].set_symbol(ch).set_style(style);
        }
    }
}
