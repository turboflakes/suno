use crate::error::{Error, ResultExt};
use subxt::client::{ClientAtBlock, OnlineClientAtBlockImpl};
use subxt::ext::codec::{Compact, Decode, Encode};
use subxt::utils::AccountId32;
use subxt::Metadata;
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
    let (extra, additional) = encode_extensions(api, signer, &genesis_hash.0).await?;
    let mut extensions = extra;
    extensions.extend_from_slice(&additional);
    let data = transaction(&signer.0, &bytes_encoded, &extensions, &genesis_hash.0);
    Ok(data)
}

/// Build a fully SCALE-encoded, ready-to-submit V4 "Signed" extrinsic from a signature
/// obtained from Polkadot Vault.
///
/// This deliberately doesn't go through subxt's typed `TransactionExtensions` machinery
/// (`CustomConfig::TransactionExtensions`), because that's a compile-time-fixed set of
/// extensions that doesn't match this chain's actual (metadata-driven, and occasionally
/// exotic) extension list — see `encode_extensions` below, which is what Vault was shown
/// and what actually signed the transaction. Reusing that same computation here guarantees
/// the bytes we submit match the bytes Vault signed.
pub async fn build_signed_extrinsic(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    signer: &AccountId32,
    call_data_bytes: &[u8],
    signature: &[u8; 64],
) -> Result<Vec<u8>, Error> {
    let genesis_hash = api.genesis_hash().ok_or(Error::GenesisHashNotAvailable)?;
    let (extra, _additional) = encode_extensions(api, signer, &genesis_hash.0).await?;
    Ok(signed_extrinsic_bytes(
        &signer.0,
        signature,
        &extra,
        call_data_bytes,
    ))
}

/// Assemble the final SCALE-encoded V4 "Signed" extrinsic, given the same `extra` bytes
/// that were embedded in the transaction QR (see `encode_extensions`).
fn signed_extrinsic_bytes(
    signer: &[u8; 32],
    signature: &[u8; 64],
    extra: &[u8],
    call_data_bytes: &[u8],
) -> Vec<u8> {
    let mut body = Vec::new();
    body.push(0x84); // is_signed (0x80) + extrinsic version 4
    body.push(0x00); // MultiAddress::Id
    body.extend_from_slice(signer);
    body.push(0x01); // MultiSignature::Sr25519
    body.extend_from_slice(signature);
    body.extend_from_slice(extra);
    body.extend_from_slice(call_data_bytes); // raw call bytes, no length prefix (it's the last field)

    let mut out = Vec::new();
    Compact(body.len() as u32).encode_to(&mut out);
    out.extend_from_slice(&body);
    out
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

async fn encode_extensions(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    signer: &AccountId32,
    genesis_hash_bytes: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let nonce = api.tx().account_nonce(signer).await.boxed()?;
    let spec_version = api.spec_version();
    let tx_version = api.transaction_version();

    // Vault only ever sees V14 metadata (that's what gets uploaded to it via the "load
    // metadata" QR flow, since Vault's parser doesn't understand the multiple
    // transaction-extension "pipeline versions" that V15/V16 metadata can describe for
    // newer V5 "General" transactions). Build the extra/additional bytes from that same
    // V14 view instead of the light client's natively-negotiated metadata, so what we
    // encode always matches Vault's single, flat, legacy-compatible extension list.
    let metadata = fetch_v14_metadata(api).await?;
    let extensions: Vec<_> = metadata
        .extrinsic()
        .transaction_extensions_to_use_for_encoding()
        .collect();
    tracing::info!(
        "transaction extensions used for encoding: {:?}",
        extensions
            .iter()
            .map(|e| e.identifier())
            .collect::<Vec<_>>()
    );

    Ok(encode_extension_values(
        extensions,
        nonce,
        spec_version,
        tx_version,
        genesis_hash_bytes,
    ))
}

/// Encode the "extra" (in-extrinsic) and "additional" (signed-but-not-sent) bytes for a
/// given set of transaction extensions. Pure and synchronous so it can be exercised
/// directly in tests, without needing a live chain connection.
fn encode_extension_values<'a>(
    extensions: impl IntoIterator<Item = subxt::metadata::TransactionExtensionMetadata<'a>>,
    nonce: u64,
    spec_version: u32,
    tx_version: u32,
    genesis_hash_bytes: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    let mut data = Vec::new(); //  "extra" data, sent inside the extrinsic and needs to be signed by the user
    let mut additional_part = Vec::new(); // additional data, needs to be signed but not sent

    for ext in extensions {
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

    (data, additional_part)
}

/// Fetch metadata downgraded to V14, the same view Vault receives via the
/// "load metadata" QR flow (see `fetch_metadata` in each chain's `runtime_apis.rs`).
async fn fetch_v14_metadata(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
) -> Result<Metadata, Error> {
    let args = 14u32.encode();
    let bytes = api
        .runtime_apis()
        .call_raw("Metadata_metadata_at_version", Some(&args))
        .await
        .boxed()?;
    let opaque: Option<Vec<u8>> = Decode::decode(&mut &bytes[..])?;
    let metadata_bytes = opaque.ok_or(Error::MetadataV14NotAvailable)?;
    let metadata = Metadata::decode_from(&metadata_bytes)?;
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compact_len_prefix(n: usize) -> Vec<u8> {
        Compact(n as u32).encode()
    }

    /// `build_transaction_qrcode` (what Vault is shown and signs) and `build_signed_extrinsic`
    /// (what we submit once Vault hands the signature back) both derive their "extra" bytes
    /// from `encode_extension_values`. This pins that down: given the same extension list and
    /// inputs, the `extra` bytes embedded in the QR content and in the final extrinsic must be
    /// byte-identical. If someone changes one side to use a different extension source (e.g.
    /// subxt's compile-time-fixed `TransactionExtensions` tuple) without the other, this fails.
    #[test]
    fn vault_signed_payload_and_submitted_extrinsic_use_the_same_extra_bytes() {
        let metadata_bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../chains/asset-hub-polkadot/artifacts/metadata/asset_hub_polkadot_metadata_small.scale"
        ));
        let metadata = Metadata::decode_from(metadata_bytes).expect("decode bundled metadata");

        let extensions: Vec<_> = metadata
            .extrinsic()
            .transaction_extensions_to_use_for_encoding()
            .collect();
        assert!(
            !extensions.is_empty(),
            "fixture metadata should declare at least one transaction extension"
        );

        let nonce = 7u64;
        let spec_version = 2_005_000u32;
        let tx_version = 15u32;
        let genesis_hash = [0xAAu8; 32];
        let signer = [0x11u8; 32];
        let signature = [0x22u8; 64];
        let call_data = b"call-data-\x01\x02\x03".to_vec(); // arbitrary short "raw call" bytes

        let (extra, additional) =
            encode_extension_values(extensions, nonce, spec_version, tx_version, &genesis_hash);
        assert!(!extra.is_empty(), "expected some extra bytes to encode");

        // What Vault is shown and signs. `transaction()` expects the call data to already
        // be Compact<u32>-length-prefixed, matching `build_transaction_qrcode`'s real usage.
        let mut signing_extensions = extra.clone();
        signing_extensions.extend_from_slice(&additional);
        let call_data_encoded = call_data.encode();
        let qr_content = transaction(
            &signer,
            &call_data_encoded,
            &signing_extensions,
            &genesis_hash,
        );

        // What we submit after getting Vault's signature back:
        let extrinsic_bytes = signed_extrinsic_bytes(&signer, &signature, &extra, &call_data);

        // Locate `extra` inside the QR content: frame(5) + prelude(3) + pubkey(32) +
        // Compact<u32> call length + call_data, then `extra` starts.
        let call_len_prefix = compact_len_prefix(call_data.len());
        let qr_extra_offset = 5 + 3 + 32 + call_len_prefix.len() + call_data.len();
        let qr_extra = &qr_content[qr_extra_offset..qr_extra_offset + extra.len()];

        // Locate `extra` inside the submitted extrinsic: Compact<u32> body length +
        // version(1) + address variant(1) + pubkey(32) + sig variant(1) + signature(64),
        // then `extra` starts.
        let body_len = 1 + 1 + 32 + 1 + 64 + extra.len() + call_data.len();
        let body_len_prefix_len = compact_len_prefix(body_len).len();
        let extrinsic_extra_offset = body_len_prefix_len + 1 + 1 + 32 + 1 + 64;
        let extrinsic_extra =
            &extrinsic_bytes[extrinsic_extra_offset..extrinsic_extra_offset + extra.len()];

        assert_eq!(
            qr_extra, extrinsic_extra,
            "the extra bytes embedded in what Vault signs must match the extra bytes embedded in what is submitted"
        );
        assert_eq!(qr_extra, extra.as_slice());
    }
}
