use crate::error::{Error, ResultExt};
use subxt::ext::codec::{Compact, Encode};
use subxt::utils::AccountId32;
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    SubstrateConfig,
};

/// Build a transaction QR code according to the UOS spec:
/// https://github.com/novasamatech/parity-signer/blob/master/docs/src/development/UOS.md
pub async fn build_qrcode(
    api: &ClientAtBlock<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    signer: &AccountId32, // signer's account ID
    bytes: &[u8],         // raw call data bytes
) -> Result<Vec<u8>, Error> {
    let bytes_encoded = bytes.encode();
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
    api: &ClientAtBlock<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
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
            _ => {} // CheckNonZeroSender and others have empty ty + additional_signed
        }
    }

    data.extend_from_slice(&additional_part);
    Ok(data)
}
