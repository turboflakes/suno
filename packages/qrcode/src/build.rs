use crate::error::Error;
use subxt::client::{ClientAtBlock, OnlineClientAtBlockImpl};
use subxt::ext::codec::Encode;
use subxt::utils::AccountId32;
use suno_config::{transactions::encode_extensions, CustomConfig, SupportedRuntime};

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
    let (extra, additional) = encode_extensions(api, signer, &genesis_hash.0)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;
    let mut extensions = extra;
    extensions.extend_from_slice(&additional);
    let data = transaction(&signer.0, &bytes_encoded, &extensions, &genesis_hash.0);
    Ok(data)
}

fn transaction(
    signer: &[u8; 32],        // signer's public key
    call_data_encoded: &[u8], // call data, Compact<u32>-length-prefixed (Vault splits method from
    // extensions using this length prefix — see `cut_method_extensions` in Vault's parser)
    extensions_encoded: &[u8], // SCALE-encoded extensions
    genesis_hash: &[u8; 32],
) -> Vec<u8> {
    let mut content = Vec::new();

    // 3-byte prelude: Substrate + Sr25519 + transaction
    content.extend_from_slice(&[0x53, 0x01, 0x02]);

    // signer public key
    content.extend_from_slice(signer);

    // call_data_encoded (already SCALE-encoded, length-prefixed)
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

#[cfg(test)]
mod tests {
    use super::*;
    use subxt::ext::codec::Compact;

    /// `transaction()` packs the QR content as prelude + pubkey + Compact<u32>-length-prefixed
    /// call + extensions blob + trailing genesis hash. This pins down that layout regardless of
    /// where the extensions bytes come from (see `suno_config::transactions` for that).
    #[test]
    fn transaction_embeds_call_and_extensions_at_the_expected_offsets() {
        let signer = [0x11u8; 32];
        let genesis_hash = [0xAAu8; 32];
        let call_data = b"call-data-\x01\x02\x03".to_vec();
        let call_data_encoded = call_data.encode();
        let extensions = vec![0xDEu8, 0xAD, 0xBE, 0xEF];

        let content = transaction(&signer, &call_data_encoded, &extensions, &genesis_hash);

        // frame(5) + prelude(3) + pubkey(32)
        let mut offset = 5 + 3;
        assert_eq!(&content[offset..offset + 32], &signer);
        offset += 32;

        assert_eq!(&content[offset..offset + call_data_encoded.len()], &call_data_encoded[..]);
        offset += call_data_encoded.len();

        assert_eq!(&content[offset..offset + extensions.len()], &extensions[..]);
        offset += extensions.len();

        assert_eq!(&content[offset..offset + 32], &genesis_hash);
        offset += 32;

        assert_eq!(content.len(), offset);

        // Sanity-check the length prefix itself matches a plain `Vec<u8>` encoding of call_data.
        assert_eq!(call_data_encoded, {
            let mut v = Compact(call_data.len() as u32).encode();
            v.extend_from_slice(&call_data);
            v
        });
    }
}
