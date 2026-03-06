use scale_encode::{EncodeAsFields, Error as EncodeError, FieldIter, TypeResolver};
use subxt::{
    transactions::{Payload, ValidationDetails},
    OnlineClient, SubstrateConfig,
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
    pub async fn from_call_data(
        api: &OnlineClient<SubstrateConfig>,
        call_data: Bytes,
    ) -> Result<Self, Error> {
        let at_block = api.at_current_block().await?;
        let metadata = at_block.metadata();
        let pallet = metadata
            .pallet_by_call_index(call_data[0])
            .ok_or(Error::PalletNotFound)?;
        let call_variant = pallet
            .call_variant_by_index(call_data[1])
            .ok_or(Error::CallNotFound)?;

        Ok(Self {
            pallet_name: pallet.name().to_string(),
            call_name: call_variant.name.clone(),
            field_bytes: RawFields(call_data[2..].to_vec()),
        })
    }
}

/// Suno specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OnlineClient error: {0}")]
    OnlineClient(#[from] subxt::error::OnlineClientAtBlockError),
    #[error("Pallet not found")]
    PalletNotFound,
    #[error("Call not found")]
    CallNotFound,
    #[error("Other error: {0}")]
    Other(String),
}
