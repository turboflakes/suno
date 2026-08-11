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
const DISPLAY_H: u32 = 640;

pub struct Scanner {
    camera: Camera,
    /// Reused across frames to avoid per-frame allocation.
    reader: QRCodeReader,
}

impl Scanner {
    pub fn new() -> Result<Self, Error> {
        let index = CameraIndex::Index(0);
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
        // AVFoundation logs a harmless "AVCaptureDeviceTypeExternal is deprecated"
        // notice to stderr while nokhwa enumerates devices on macOS. Until nokhwa removes the
        // warning, silence stderr for the duration of the call.
        let camera = silence_stderr(|| Camera::new(index, requested))?;
        Ok(Self {
            camera,
            reader: QRCodeReader,
        })
    }

    pub fn open(&mut self) -> Result<(), Error> {
        silence_stderr(|| self.camera.open_stream())?;
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
        let display = display.fliph();
        let data = self.decode(qr);

        Ok((data, display))
    }
}

/// Run `f` with the process's stderr temporarily redirected to `/dev/null`.
///
/// Used to hide macOS's noisy (and harmless) AVFoundation deprecation
/// warning — `AVCaptureDeviceTypeExternal is deprecated for Continuity
/// Cameras` — that nokhwa's vendored bindings trigger on every camera
/// enumeration and that we can't suppress at the source.
#[cfg(target_os = "macos")]
fn silence_stderr<T>(f: impl FnOnce() -> T) -> T {
    use std::ffi::CString;
    use std::os::unix::io::RawFd;

    const STDERR_FD: RawFd = 2;

    // SAFETY: standard POSIX fd juggling — duplicate stderr so it can be
    // restored, then point fd 2 at /dev/null for the duration of `f`.
    let saved_fd = unsafe { libc::dup(STDERR_FD) };
    if saved_fd < 0 {
        return f();
    }

    let devnull_path = CString::new("/dev/null").unwrap();
    let devnull_fd = unsafe { libc::open(devnull_path.as_ptr(), libc::O_WRONLY) };
    if devnull_fd >= 0 {
        unsafe {
            libc::dup2(devnull_fd, STDERR_FD);
            libc::close(devnull_fd);
        }
    }

    let result = f();

    unsafe {
        libc::dup2(saved_fd, STDERR_FD);
        libc::close(saved_fd);
    }

    result
}

#[cfg(not(target_os = "macos"))]
fn silence_stderr<T>(f: impl FnOnce() -> T) -> T {
    f()
}
