use super::proxy::proxy;
use log::info;
use suno_common::actions::{Action, TxAction};
use suno_common::errors::SunoError;
use suno_common::file::get_keypair_from_seed_file;
use subxt::ext::scale_value::Composite;
use subxt::ext::scale_value::ValueDef;
use subxt::{
    dynamic::Value, error::TransactionError, tx::DynamicPayload, tx::TxStatus, utils::AccountId32,
    OnlineClient, SubstrateConfig,
};
use subxt_signer::sr25519::Keypair;
use tokio::sync::mpsc::UnboundedSender;

pub async fn submit_as_proxy(
    api: &OnlineClient<SubstrateConfig>,
    payload: DynamicPayload,
    proxied_account: AccountId32,
    tx: UnboundedSender<Action>,
) -> Result<(), SunoError> {
    let proxy_signer: Keypair = get_keypair_from_seed_file(None)?;
    let proxy_payload = proxy(proxied_account.clone(), payload);
    submit(api, proxy_payload, proxy_signer, tx).await
}

async fn submit(
    api: &OnlineClient<SubstrateConfig>,
    payload: DynamicPayload,
    signer: Keypair,
    tx: UnboundedSender<Action>,
) -> Result<(), SunoError> {
    let mut response = api
        .tx()
        .sign_and_submit_then_watch_default(&payload, &signer)
        .await?;

    while let Some(status) = response.next().await {
        match status? {
            TxStatus::Broadcasted => {
                let _ = tx.send(Action::Transaction(TxAction::Broadcasting));
            }
            TxStatus::InBestBlock(_) => {
                let _ = tx.send(Action::Transaction(TxAction::InBestBlock));
            }
            TxStatus::InFinalizedBlock(in_block) => {
                let _ = tx.send(Action::Transaction(TxAction::InFinalizedBlock));
                info!(
                    "Transaction {:?} is finalized in block {:?}",
                    in_block.extrinsic_hash(),
                    in_block.block_hash()
                );

                let _ = in_block.wait_for_success().await?;

                let events = in_block.fetch_events().await?;
                for event in events.iter() {
                    let event = event?;
                    let pallet = event.pallet_name();
                    let variant = event.variant_name();

                    if pallet == "Proxy" && variant == "ProxyExecuted" {
                        let field_values = event.field_values()?;
                        field_values.values().for_each(|result_value| {
                            if let Ok(Some((index, error))) = extract_module_error(result_value) {
                                let metadata = api.metadata();
                                if let Some(pallet) = metadata.pallet_by_index(index.into()) {
                                    if let Some(error_index) = error.first() {
                                        if let Some(error) =
                                            pallet.error_variant_by_index(*error_index)
                                        {
                                            info!("Error: {:?}", error.docs.join("\n"));
                                        }
                                    }
                                }
                            };
                        });
                    }
                }

                let _ = tx.send(Action::Transaction(TxAction::Success));
            }
            TxStatus::Error { message } => return Err(TransactionError::Error(message).into()),
            TxStatus::Invalid { message } => return Err(TransactionError::Invalid(message).into()),
            TxStatus::Dropped { message } => return Err(TransactionError::Dropped(message).into()),

            _ => {}
        }
    }
    Ok(())
}

// Helper function to extract module index and error data from a Value
fn extract_module_error(value: &Value<u32>) -> Result<Option<(u8, Vec<u8>)>, SunoError> {
    match value {
        Value::<_> { value, context: _ } => {
            info!("Value: {}", value);
            match &value {
                ValueDef::Variant(variant) if variant.name == "Ok" => {
                    info!("Proxy call succeeded!");
                    return Ok(None);
                }
                ValueDef::Variant(variant) if variant.name == "Err" => {
                    let variant_values = variant.values.clone();
                    match variant_values {
                        Composite::Unnamed(values) => {
                            for value in values {
                                if let Ok(result) = extract_module_error(&value) {
                                    return Ok(result);
                                }
                            }
                        }
                        _ => return Err("Invalid variant".into()),
                    }
                }
                ValueDef::Variant(variant) if variant.name == "Module" => {
                    let variant_values = variant.values.clone();
                    match variant_values {
                        Composite::Unnamed(values) => {
                            for value in values {
                                if let Ok(result) = extract_module_error(&value) {
                                    return Ok(result);
                                }
                            }
                        }
                        _ => return Err("Invalid variant".into()),
                    }
                }
                ValueDef::Composite(composite) => {
                    let mut module_index: Option<u8> = None;
                    let mut error_data: Option<Vec<u8>> = None;

                    match composite {
                        Composite::Named(fields) => {
                            for (name, value) in fields {
                                if name == "index" {
                                    module_index = extract_number(&value).map(|n| n as u8);
                                } else if name == "error" {
                                    error_data = extract_error_data(&value);
                                }
                            }
                        }
                        _ => return Err("Invalid variant".into()),
                    }
                    info!("Proxy fields result! {:?} {:?}", module_index, error_data);
                    if let (Some(index), Some(error)) = (module_index, error_data) {
                        return Ok(Some((index, error)));
                    }
                }
                // Other cases
                _ => {}
            }
        }
    }

    Err(format!("Could not extract module error from value: {:?}", value).into())
}

fn extract_number(value: &Value<u32>) -> Option<u128> {
    match value {
        Value::<_> { value, context: _ } => match &value {
            ValueDef::Primitive(primitive) => match primitive {
                subxt::ext::scale_value::Primitive::U128(n) => Some(*n),
                _ => None,
            },
            _ => None,
        },
    }
}

fn extract_error_data(value: &Value<u32>) -> Option<Vec<u8>> {
    match value {
        Value::<_> { value, context: _ } => match &value {
            ValueDef::Composite(composite) => match composite {
                Composite::Unnamed(values) => {
                    let mut error_data = Vec::new();
                    for value in values {
                        if let Some(value) = extract_number(value).map(|n| n as u8) {
                            error_data.push(value);
                        };
                    }
                    Some(error_data)
                }
                _ => None,
            },
            _ => None,
        },
    }
}
