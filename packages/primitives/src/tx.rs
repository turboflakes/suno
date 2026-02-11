use subxt::{error::CoreExtrinsicError, tx::Payload, Metadata};

pub type Bytes = Vec<u8>;

// Helper to wrap raw bytes into a type Subxt can submit
pub fn payload_from_bytes(bytes: Bytes) -> impl Payload {
    struct RawData(Bytes);
    impl Payload for RawData {
        fn encode_call_data_to(
            &self,
            _metadata: &Metadata,
            out: &mut Bytes,
        ) -> Result<(), CoreExtrinsicError> {
            out.extend_from_slice(&self.0);
            Ok(())
        }
    }
    RawData(bytes)
}
