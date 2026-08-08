pub mod build;
pub mod error;
pub mod layout;
pub mod scanner;
pub mod widgets;

pub type QrBytes = Vec<u8>;
pub use build::NetworkSpecsToSend;
pub use widgets::{
    metadata::{MetadataState, MetadataWidget},
    qrcode::QrCodeWidget,
    scanner::ScannerWidget,
};
