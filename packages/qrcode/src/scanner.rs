use crate::error::Error;
use image::DynamicImage;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use rxing::{
    common::HybridBinarizer, qrcode::QRCodeReader, BinaryBitmap, BufferedImageLuminanceSource,
    Reader,
};

/// Max display size sent to the UI thread — keep small for fast encoding.
const DISPLAY_W: u32 = 640;
const DISPLAY_H: u32 = 480;

pub struct Scanner {
    camera: Camera,
    /// Reused across frames to avoid per-frame allocation.
    reader: QRCodeReader,
}

impl Scanner {
    pub fn new() -> Result<Self, Error> {
        let index = CameraIndex::Index(0);
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let camera = Camera::new(index, requested)?;
        Ok(Self {
            camera,
            reader: QRCodeReader,
        })
    }

    pub fn open(&mut self) -> Result<(), Error> {
        self.camera.open_stream()?;
        Ok(())
    }

    /// Capture one camera frame, cropped to the largest centered square.
    fn capture_square(&mut self) -> Result<DynamicImage, Error> {
        let frame = self.camera.frame()?;
        let full = DynamicImage::ImageRgb8(frame.decode_image::<RgbFormat>()?);

        let side = full.width().min(full.height());
        let x = (full.width() - side) / 2;
        let y = (full.height() - side) / 2;
        Ok(full.crop_imm(x, y, side, side))
    }

    /// Decode a QR code from `image`. Returns `None` when none is found
    /// (any decode error is treated as "no QR in frame").
    fn decode(&mut self, image: DynamicImage) -> Option<Vec<u8>> {
        let source = BufferedImageLuminanceSource::new(image);
        let mut bitmap = BinaryBitmap::new(HybridBinarizer::new(source));

        let result = self.reader.decode(&mut bitmap).ok()?;
        let qr_text = result.getText();
        hex::decode(result.getText().strip_prefix("0x").unwrap_or(qr_text)).ok()
    }

    pub fn scan_frame(&mut self) -> Result<(Option<Vec<u8>>, DynamicImage), Error> {
        let qr = self.capture_square()?;
        let display = qr.thumbnail(DISPLAY_W, DISPLAY_H);
        let data = self.decode(qr);

        Ok((data, display))
    }
}
