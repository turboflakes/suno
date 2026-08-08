use crate::error::{Error, ResultExt};
use subxt::client::{ClientAtBlock, OnlineClientAtBlockImpl};
use subxt::ext::codec::{Compact, Encode};
use subxt::utils::AccountId32;
use suno_config::{CustomConfig, SupportedRuntime};

#[derive(Encode)]
#[repr(u8)]
pub enum Encryption {
    Ed25519 = 0,
    Sr25519 = 1,
    Ecdsa = 2,
}

/// The UOS spec for `0xc1` (add specs) expects the payload to be
/// a SCALE-encoded `NetworkSpecsToSend` struct
#[derive(Encode)]
pub struct NetworkSpecsToSend {
    base58prefix: u16, // e.g. 0
    color: String,
    decimals: u8,           // e.g. 10
    encryption: Encryption, // e.g. Sr25519
    genesis_hash: [u8; 32],
    logo: String,
    name: String,    // e.g. "Polkadot"
    path_id: String, // e.g. "//polkadot"
    secondary_color: String,
    title: String,
    unit: String, // tokenSymbol = "DOT"
}

impl NetworkSpecsToSend {
    /// Returns the SCALE-encoded payload for this network specs update.
    pub fn payload(&self) -> Vec<u8> {
        self.encode()
    }
}

impl From<SupportedRuntime> for NetworkSpecsToSend {
    fn from(runtime: SupportedRuntime) -> Self {
        Self {
            base58prefix: runtime.account_format(),
            color: "".to_string(),
            decimals: runtime.token_decimals() as u8,
            encryption: Encryption::Sr25519,
            genesis_hash: runtime.chain_genesis_hash().into(),
            logo: "".to_string(),
            name: runtime.legacy_name(),
            path_id: format!("//{}", runtime.legacy_name()),
            secondary_color: "".to_string(),
            title: "".to_string(),
            unit: runtime.token_symbol().to_string(),
        }
    }
}

/// Build a chain-specs signed QR code according to the UOS spec:
/// https://github.com/novasamatech/parity-signer/blob/master/docs/src/development/UOS.md
pub fn build_chain_specs_qrcode_signed(
    payload: &[u8],
    public_key: &[u8],
    signature: &[u8],
) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&[0x53, 0x01, 0xc1]); // 3-byte prelude: Substrate + Sr25519 + add specs update
    data.extend_from_slice(public_key); // 32 bytes
    data.extend_from_slice(&payload.encode()); // SCALE-encoded data
    data.extend_from_slice(signature); // 64 bytes
    data
}

/// Build a chain-specs unsigned QR code according to the UOS spec:
/// https://github.com/novasamatech/parity-signer/blob/master/docs/src/development/UOS.md
pub fn build_chain_specs_qrcode_unsigned(payload: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&[0x53, 0xff, 0xc1]); // 3-byte prelude: Substrate + unsigned + add specs update
    data.extend_from_slice(&payload.encode()); // SCALE-encoded data
    data
}

/// Build a metadata signed QR code according to the UOS spec:
/// https://github.com/novasamatech/parity-signer/blob/master/docs/src/development/UOS.md
pub fn build_metadata_qrcode_signed(
    metadata_bytes: &[u8],
    genesis_hash: &[u8; 32],
    public_key: &[u8],
    signature: &[u8],
) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&[0x53, 0x01, 0x80]); // 3-byte prelude: Substrate + Sr25519 + load metadata update
    data.extend_from_slice(public_key); // 32 bytes
    data.extend_from_slice(&metadata_bytes.encode());
    data.extend_from_slice(genesis_hash);
    data.extend_from_slice(signature); // 64 bytes
    data
}

/// Build a metadata unsigned QR code according to the UOS spec:
///
/// payload is the raw `meta`-prefixed blob returned by `Metadata_metadata_at_version`;
/// https://github.com/novasamatech/parity-signer/blob/master/docs/src/development/UOS.md
pub fn build_metadata_qrcode_unsigned(metadata_bytes: &[u8], genesis_hash: &[u8; 32]) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&[0x53, 0xff, 0x80]); // 3-byte prelude: Substrate + unsigned + load metadata update
    data.extend_from_slice(&metadata_bytes.encode());
    data.extend_from_slice(genesis_hash);
    data
}

/// Build the `load_metadata` update content that must be signed and transferred.
/// The metadata itself is the raw `meta`-prefixed blob returned by
/// `Metadata_metadata_at_version`; UOS wraps it as a SCALE `Vec<u8>`.
pub fn build_metadata_payload(metadata_bytes: &[u8], genesis_hash: &[u8; 32]) -> Vec<u8> {
    let mut data = metadata_bytes.encode();
    data.extend_from_slice(genesis_hash);
    data
}

/// Build a transaction QR code according to the UOS spec:
/// https://github.com/novasamatech/parity-signer/blob/master/docs/src/development/UOS.md
pub async fn build_transaction_qrcode(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    signer: &AccountId32,   // signer's account ID
    call_data_bytes: &[u8], // raw call data bytes
) -> Result<Vec<u8>, Error> {
    let bytes_encoded = call_data_bytes.encode();
    let genesis_hash = api.genesis_hash().ok_or(Error::GenesisHashNotAvailable)?;
    let extensions = encode_extensions(api, signer, &genesis_hash.0).await?;
    let data = transaction(&signer.0, &bytes_encoded, &extensions, &genesis_hash.0);
    Ok(data)
}

fn transaction(
    signer: &[u8; 32],         // signer's public key
    call_data_encoded: &[u8],  // SCALE-encoded call bytes
    extensions_encoded: &[u8], // SCALE-encoded extensions
    genesis_hash: &[u8; 32],
) -> Vec<u8> {
    let mut content = Vec::new();

    // 3-byte prelude: Substrate + Sr25519 + transaction
    content.extend_from_slice(&[0x53, 0x01, 0x02]);

    // signer public key
    content.extend_from_slice(signer);

    // call_data_encoded (already SCALE-encoded)
    content.extend_from_slice(call_data_encoded);

    // extensions (already SCALE-encoded)
    content.extend_from_slice(extensions_encoded);

    // genesis hash (always last, fixed 32 bytes)
    content.extend_from_slice(genesis_hash);

    // wrap in single-frame legacy multiframe envelope
    wrap_single_frame(&content)
}

fn wrap_single_frame(content: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(5 + content.len());
    frame.push(0x00); // legacy multiframe
    frame.extend_from_slice(&1u16.to_be_bytes()); // frame_count = 1
    frame.extend_from_slice(&0u16.to_be_bytes()); // frame_index = 0
    frame.extend_from_slice(content);
    frame
}

async fn encode_extensions(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    signer: &AccountId32,
    genesis_hash_bytes: &[u8],
) -> Result<Vec<u8>, Error> {
    let nonce = api.tx().account_nonce(signer).await.boxed()?;
    let spec_version = api.spec_version();
    let tx_version = api.transaction_version();

    let mut data = Vec::new(); //  "extra" data, sent inside the extrinsic and needs to be signed by the user
    let mut additional_part = Vec::new(); // additional data, needs to be signed but not sent

    for ext in api
        .metadata()
        .extrinsic()
        .transaction_extensions_to_use_for_encoding()
    {
        match ext.identifier() {
            "CheckMortality" | "CheckEra" => {
                data.push(0x00); // Era::Immortal
                additional_part.extend_from_slice(genesis_hash_bytes); // block_hash = genesis_hash for immortal
            }
            "CheckNonce" => {
                data.extend_from_slice(&Compact(nonce).encode());
            }
            "ChargeTransactionPayment" => {
                data.extend_from_slice(&Compact(0u128).encode()); // tip = 0
            }
            "ChargeAssetTxPayment" => {
                data.extend_from_slice(&Compact(0u128).encode()); // tip = 0
                data.push(0x00); // Option<AssetId>::None
            }
            "CheckMetadataHash" => {
                data.push(0x00); // Mode::Disabled
                additional_part.push(0x00); // None (no hash)
            }
            "CheckSpecVersion" => {
                additional_part.extend_from_slice(&spec_version.encode());
            }
            "CheckTxVersion" => {
                additional_part.extend_from_slice(&tx_version.encode());
            }
            "CheckGenesis" => {
                additional_part.extend_from_slice(genesis_hash_bytes);
            }
            "AuthorizeValueTransfer" => {
                data.push(0x00); // Option<[u8; 64]>::None
            }
            "AsPgas" => {
                data.push(0x00); // Option<AsPgasInfo>::None
            }
            "AsRingAlias" => {
                data.push(0x00); // Option<AsRingAliasInfo>::None
            }
            "AsDotnsGateway" => {
                data.push(0x00); // Option<AsDotnsGatewayInfo>::None
            }
            "AuthorizeCall" => {
                // no value encoded
            }
            "RestrictOrigins" => {
                data.push(0x01); // bool = true (restricted)
            }
            _ => {} // CheckNonZeroSender and others have empty ty + additional_signed
        }
    }

    data.extend_from_slice(&additional_part);
    Ok(data)
}
