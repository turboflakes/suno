use crate::widgets::qrcode::render_qr;
use qrcodegen::{QrCode, QrCodeEcc};
use raptorq::{Encoder, ObjectTransmissionInformation};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};
const CHUNK_SIZE: u16 = 768; // bytes per QR frame
const QUIET: i32 = 4;

/// Manages the state of a QR code metadata widget, including the current frame and the list of frames.
pub struct MetadataState {
    frames: Vec<QrCode>, // metadata QR code frames
    frame_index: usize,  // current frame
}

impl MetadataState {
    pub fn new(payload: &[u8]) -> Self {
        // Build fountain-code packets
        let config = ObjectTransmissionInformation::with_defaults(payload.len() as u64, CHUNK_SIZE);
        let encoder = Encoder::new(payload, config);

        let size = payload.len();

        // Match Polkadot Vault's encoder: one recovery packet per source packet.
        let repair_packets = if size <= CHUNK_SIZE as usize {
            0
        } else {
            (size / CHUNK_SIZE as usize) as u32
        };

        // Generate QR codes from the packets
        let frames: Vec<QrCode> = encoder
            .get_encoded_packets(repair_packets)
            .into_iter()
            .filter_map(|pkt| {
                let mut chunk = Vec::new();
                let header = 0x80000000u32 | size as u32;
                chunk.extend_from_slice(&header.to_be_bytes()); // 4-byte header
                chunk.extend_from_slice(&pkt.serialize());
                QrCode::encode_binary(&chunk, QrCodeEcc::Low).ok()
            })
            .collect();

        Self {
            frames,
            frame_index: 0,
        }
    }

    pub fn frame(&self) -> Option<&QrCode> {
        self.frames.get(self.frame_index)
    }

    pub fn advance_frame(&mut self) {
        if !self.frames.is_empty() {
            self.frame_index = (self.frame_index + 1) % self.frames.len();
        }
    }

    pub fn frame_index(&self) -> usize {
        self.frame_index
    }

    pub fn total_frames(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

/// Renders a QR code metadata frame
pub struct MetadataWidget<'a> {
    qr: &'a QrCode,
    block: Option<Block<'a>>,
    style: Style,
}

impl<'a> MetadataWidget<'a> {
    pub fn new(qr: &'a QrCode) -> Self {
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
        (self.qr.size() + QUIET * 2) as u16
    }

    pub fn height(&self) -> u16 {
        let w = self.width() as i32;
        ((w + 1) / 2) as u16
    }
}

impl Widget for MetadataWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(b) => {
                b.clone().render(area, buf);
                b.inner(area)
            }
            None => area,
        };

        render_qr(self.qr, inner, self.style, buf);
    }
}
