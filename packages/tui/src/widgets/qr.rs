use qrcodegen::{QrCode, QrCodeEcc};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};

pub struct Qr<'a> {
    data: &'a str,
    block: Option<Block<'a>>,
    ecc: QrCodeEcc,
}

impl<'a> Qr<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            data,
            block: None,
            ecc: QrCodeEcc::Low,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn ecc(mut self, ecc: QrCodeEcc) -> Self {
        self.ecc = ecc;
        self
    }
}

impl Widget for Qr<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        let qr = match QrCode::encode_text(self.data, self.ecc) {
            Ok(q) => q,
            Err(_) => return,
        };

        // qrcodegen's get_module() returns false (white) for any out-of-bounds
        // coordinate, so iterating -QUIET..size+QUIET gives the quiet zone for free.
        const QUIET: i32 = 4;
        let size = qr.size(); // i32, symbol modules only (no quiet zone)
        let padded = size + QUIET * 2; // total columns/rows to render

        // One style for all cases: fg = dark (Black), bg = light (White).
        // The block character encodes which half is dark or light:
        //   ▀  top = fg,    bottom = bg
        //   ▄  bottom = fg, top    = bg
        //   █  all = fg
        //   ' ' all = bg
        let style = Style::default().fg(Color::Black).bg(Color::White);

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
                buf[(x, y)].set_symbol(ch).set_style(style);
            }
        }
    }
}
