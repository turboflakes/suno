use super::node_runtime;
use log::info;
use node_runtime::proxy::events::ProxyExecuted;
use node_runtime::runtime_types::asset_hub_polkadot_runtime::ProxyType;
use node_runtime::runtime_types::sp_runtime::DispatchError;
use subxt::{
    error::TransactionError, tx::TxProgress, tx::TxStatus, utils::AccountId32, OnlineClient,
    SubstrateConfig,
};
use subxt_signer::sr25519::Keypair;
use suno_actions::{Action, TxAction};
use suno_error::Error;
use tokio::sync::mpsc::UnboundedSender;

type Call = node_runtime::runtime_types::asset_hub_polkadot_runtime::RuntimeCall;

pub async fn submit_as_proxy(
    api: &OnlineClient<SubstrateConfig>,
    call: Call,
    proxied_account: AccountId32,
    password: Option<String>,
    tx: UnboundedSender<Action>,
) -> Result<(), Error> {
    let proxy_signer: Keypair = suno_signer::load_keypair(password)?;

    let proxy_call =
        node_runtime::tx()
            .proxy()
            .proxy(proxied_account.into(), Some(ProxyType::Staking), call);

    let mut response = api
        .tx()
        .sign_and_submit_then_watch_default(&proxy_call, &proxy_signer)
        .await?;

    handle_response(api, &mut response, tx).await
}

async fn handle_response(
    api: &OnlineClient<SubstrateConfig>,
    response: &mut TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>,
    tx: UnboundedSender<Action>,
) -> Result<(), Error> {
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
                    //
                    if let Some(ev) = event.as_event::<ProxyExecuted>()? {
                        match ev.result {
                            Ok(_) => {
                                info!("ProxyExecuted successufuly");
                                let _ = tx.send(Action::Transaction(TxAction::Success));
                            }
                            Err(err) => match err {
                                DispatchError::Module(data) => {
                                    info!("ProxyExecuted event: {:?}", data.index);
                                    let metadata = api.metadata();
                                    if let Some(pallet) =
                                        metadata.pallet_by_index(data.index.into())
                                    {
                                        // Get the error variant based on the first byte of error_data
                                        if let Some(error_index) = data.error.first() {
                                            if let Some(error) =
                                                pallet.error_variant_by_index(*error_index)
                                            {
                                                info!("Error: {:?}", error.docs.join("\n"));
                                            }
                                        } else {
                                            return Err("Empty error data".into());
                                        }
                                    }
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
            TxStatus::Error { message } => return Err(TransactionError::Error(message).into()),
            TxStatus::Invalid { message } => return Err(TransactionError::Invalid(message).into()),
            TxStatus::Dropped { message } => return Err(TransactionError::Dropped(message).into()),

            _ => {}
        }
    }
    Ok(())
}
