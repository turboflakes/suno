use scale_encode::{EncodeAsFields, Error as EncodeError, FieldIter, TypeResolver};
use subxt::{
    transactions::{Payload, ValidationDetails},
    Metadata,
};

pub type Bytes = Vec<u8>;

pub struct RawPayload {
    pallet_name: String,
    call_name: String,
    field_bytes: RawFields,
}

pub struct RawFields(Bytes);

impl EncodeAsFields for RawFields {
    fn encode_as_fields_to<R: TypeResolver>(
        &self,
        _fields: &mut dyn FieldIter<'_, R::TypeId>,
        _types: &R,
        out: &mut Bytes,
    ) -> Result<(), EncodeError> {
        out.extend_from_slice(&self.0);
        Ok(())
    }
}

impl Payload for RawPayload {
    type CallData = RawFields;

    fn pallet_name(&self) -> &str {
        &self.pallet_name
    }
    fn call_name(&self) -> &str {
        &self.call_name
    }
    fn call_data(&self) -> &RawFields {
        &self.field_bytes
    }
    fn validation_details(&self) -> Option<ValidationDetails<'_>> {
        None
    }
}

impl RawPayload {
    pub fn from_bytes(metadata: &Metadata, bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 2 {
            return Err(Error::InvalidBytes);
        }
        let pallet = metadata
            .pallet_by_call_index(bytes[0])
            .ok_or(Error::PalletNotFound)?;
        let call_variant = pallet
            .call_variant_by_index(bytes[1])
            .ok_or(Error::CallNotFound)?;

        Ok(Self {
            pallet_name: pallet.name().to_string(),
            call_name: call_variant.name.clone(),
            field_bytes: RawFields(bytes[2..].to_vec()),
        })
    }
}

/// Suno specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Call data must be at least 2 bytes (pallet index + call index)")]
    InvalidBytes,
    #[error("Pallet not found")]
    PalletNotFound,
    #[error("Call not found")]
    CallNotFound,
}
