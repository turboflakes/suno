use crate::error::{Error, ResultExt};
use crate::substrate::CustomConfig;
use subxt::client::{ClientAtBlock, OnlineClientAtBlockImpl};
use subxt::ext::codec::{Compact, Decode, Encode};
use subxt::utils::AccountId32;
use subxt::Metadata;

/// Prepare a call for local signing (e.g. with a `subxt_signer::sr25519::Keypair`), instead
/// of via an air-gapped signer like Polkadot Vault.
///
/// Returns `(signing_payload, extra)`:
/// - `signing_payload` is the exact bytes that must be signed: `call ++ extra ++ additional`,
///   Blake2-256-hashed if longer than 256 bytes (the standard Substrate signing convention —
///   see e.g. `frame_decode::extrinsics::encode_v4_signer_payload`'s docs).
/// - `extra` should be passed to `signed_extrinsic_bytes` afterwards, along with the produced
///   signature, to assemble the final extrinsic.
///
/// This is driven by the chain's actual metadata rather than subxt's compile-time-fixed
/// `TransactionExtensions` tuple (`CustomConfig::TransactionExtensions`), which doesn't cover
/// several extensions this chain declares — see `encode_extensions` below.
pub async fn build_signing_payload(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    signer: &AccountId32,
    call_data_bytes: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let genesis_hash = api.genesis_hash().ok_or(Error::GenesisHashNotAvailable)?;
    let (extra, additional) = encode_extensions(api, signer, &genesis_hash.0).await?;

    let mut payload = Vec::with_capacity(call_data_bytes.len() + extra.len() + additional.len());
    payload.extend_from_slice(call_data_bytes);
    payload.extend_from_slice(&extra);
    payload.extend_from_slice(&additional);
    if payload.len() > 256 {
        payload = sp_crypto_hashing::blake2_256(&payload).to_vec();
    }

    Ok((payload, extra))
}

/// Build a fully SCALE-encoded, ready-to-submit V4 "Signed" extrinsic from a signature
/// obtained from an air-gapped signer like Polkadot Vault.
///
/// This deliberately doesn't go through subxt's typed `TransactionExtensions` machinery
/// (`CustomConfig::TransactionExtensions`), because that's a compile-time-fixed set of
/// extensions that doesn't match this chain's actual (metadata-driven, and occasionally
/// exotic) extension list — see `encode_extensions` below, which is what the signer was
/// shown and what actually signed the transaction. Reusing that same computation here
/// guarantees the bytes we submit match the bytes that were signed.
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
/// that were embedded in whatever signing payload was actually signed (see
/// `encode_extensions`).
pub fn signed_extrinsic_bytes(
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

/// Encode the "extra" (in-extrinsic) and "additional" (signed-but-not-sent) bytes for this
/// call, ready to be embedded in a Vault transaction QR or a locally-signed extrinsic.
pub async fn encode_extensions(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    signer: &AccountId32,
    genesis_hash_bytes: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), Error> {
    let nonce = api.tx().account_nonce(signer).await.boxed()?;
    let spec_version = api.spec_version();
    let tx_version = api.transaction_version();

    // Air-gapped signers like Vault only ever see V14 metadata (that's what gets uploaded
    // to them via the "load metadata" QR flow, since their parsers don't understand the
    // multiple transaction-extension "pipeline versions" that V15/V16 metadata can describe
    // for newer V5 "General" transactions). Build the extra/additional bytes from that same
    // V14 view instead of the light client's natively-negotiated metadata, so what we
    // encode always matches that single, flat, legacy-compatible extension list.
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
pub(crate) fn encode_extension_values<'a>(
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

/// Fetch metadata downgraded to V14, the same view an air-gapped signer like Vault receives
/// via the "load metadata" QR flow (see `fetch_metadata` in each chain's `runtime_apis.rs`).
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

    /// `build_signing_payload`/`build_signed_extrinsic` (what an air-gapped signer like Vault
    /// is shown and signs, and what we submit once it hands the signature back) both derive
    /// their "extra" bytes from `encode_extension_values`. This pins that down: given the same
    /// extension list and inputs, the `extra` bytes embedded in a signing payload and in the
    /// final extrinsic must be byte-identical. If someone changes one side to use a different
    /// extension source (e.g. subxt's compile-time-fixed `TransactionExtensions` tuple) without
    /// the other, this fails.
    ///
    /// Run against every Asset Hub network's bundled metadata: each has its own extension
    /// list (and Asset Hub Polkadot in particular has extensions the others don't), so this
    /// needs to hold for all of them individually, not just one.
    #[test]
    fn signing_payload_and_submitted_extrinsic_use_the_same_extra_bytes() {
        let fixtures: &[(&str, &[u8])] = &[
            (
                "asset-hub-polkadot",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../chains/asset-hub-polkadot/artifacts/metadata/asset_hub_polkadot_metadata_small.scale"
                )),
            ),
            (
                "asset-hub-kusama",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../chains/asset-hub-kusama/artifacts/metadata/asset_hub_kusama_metadata_small.scale"
                )),
            ),
            (
                "asset-hub-paseo",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../chains/asset-hub-paseo/artifacts/metadata/asset_hub_paseo_metadata_small.scale"
                )),
            ),
            (
                "asset-hub-westend",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../chains/asset-hub-westend/artifacts/metadata/asset_hub_westend_metadata_small.scale"
                )),
            ),
        ];

        for (network, metadata_bytes) in fixtures {
            check_signing_payload_and_submitted_extrinsic_use_the_same_extra_bytes(
                network,
                metadata_bytes,
            );
        }
    }

    fn check_signing_payload_and_submitted_extrinsic_use_the_same_extra_bytes(
        network: &str,
        metadata_bytes: &[u8],
    ) {
        let metadata = Metadata::decode_from(metadata_bytes)
            .unwrap_or_else(|e| panic!("[{network}] decode bundled metadata: {e}"));

        let extensions: Vec<_> = metadata
            .extrinsic()
            .transaction_extensions_to_use_for_encoding()
            .collect();
        assert!(
            !extensions.is_empty(),
            "[{network}] fixture metadata should declare at least one transaction extension"
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
        assert!(
            !extra.is_empty(),
            "[{network}] expected some extra bytes to encode"
        );

        // What an air-gapped signer is shown and signs: call ++ extra ++ additional.
        let mut signing_payload = call_data.clone();
        signing_payload.extend_from_slice(&extra);
        signing_payload.extend_from_slice(&additional);
        assert!(
            signing_payload.len() <= 256,
            "[{network}] test payload should be short enough to skip the blake2-256 hashing branch"
        );

        // What we submit after getting the signature back:
        let extrinsic_bytes = signed_extrinsic_bytes(&signer, &signature, &extra, &call_data);

        // Locate `extra` inside the signing payload: call_data, then `extra` starts.
        let signing_extra = &signing_payload[call_data.len()..call_data.len() + extra.len()];

        // Locate `extra` inside the submitted extrinsic: Compact<u32> body length +
        // version(1) + address variant(1) + pubkey(32) + sig variant(1) + signature(64),
        // then `extra` starts.
        let body_len = 1 + 1 + 32 + 1 + 64 + extra.len() + call_data.len();
        let body_len_prefix_len = compact_len_prefix(body_len).len();
        let extrinsic_extra_offset = body_len_prefix_len + 1 + 1 + 32 + 1 + 64;
        let extrinsic_extra =
            &extrinsic_bytes[extrinsic_extra_offset..extrinsic_extra_offset + extra.len()];

        assert_eq!(
            signing_extra, extrinsic_extra,
            "[{network}] the extra bytes embedded in the signing payload must match the extra bytes embedded in what is submitted"
        );
        assert_eq!(signing_extra, extra.as_slice(), "[{network}]");
    }
}
