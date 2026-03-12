use crate::bridge::sync::spawn_process_transaction_progress;
use log::warn;
use suno_actions::{Action, ChainAction, TxAction, ValidatorAction};
use suno_config::SupportedRuntime;
use suno_error::{Error, ResultExt};
use suno_primitives::{proxy::ProxyKey, AccountKey, Response};
use tokio::sync::mpsc::UnboundedSender;

pub fn dispatch_response_action(
    response: Response,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    match response {
        Response::Era(data) => {
            tx.send(Action::Chain(ChainAction::UpdateEra(runtime, data.value)))
                .boxed()?;
        }
        Response::Epoch(data) => {
            tx.send(Action::Chain(ChainAction::UpdateEpoch(runtime, data.value)))
                .boxed()?;
        }
        Response::TotalStaked(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalStaked(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::ActiveValidators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateActiveValidators(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::ActiveNominators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateActiveNominators(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::TotalValidators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalValidators(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::TotalNominators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalNominators(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::AuthorityStatus(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateStatus(
                account_key,
                data.value.status,
            )))
            .boxed()?;
        }
        Response::AuthorityEraPoints(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateEraPoints(
                account_key,
                data.value.points,
            )))
            .boxed()?;
        }
        Response::AuthorityPoints(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdatePoints(
                account_key,
                data.value.points,
            )))
            .boxed()?;
        }
        Response::StakeOverview(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(overview) = data.value.overview {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeOverview(
                    account_key,
                    overview,
                )))
                .boxed()?;
            } else {
                warn!("No stake overview data found for {}", account_key,);
            }
        }
        Response::StakeLedger(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(ledger) = data.value.ledger {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeLedger(
                    account_key,
                    ledger,
                )))
                .boxed()?;
            } else {
                warn!("No stake ledger data found for {}", account_key,);
            }
        }
        Response::ValidatorPrefs(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(prefs) = data.value.prefs {
                tx.send(Action::Validator(ValidatorAction::UpdateValidatorPrefs(
                    account_key,
                    prefs,
                )))
                .boxed()?;
            } else {
                warn!("No validator prefs data found for {}", account_key,);
            }
        }
        Response::ValidatorPrefsNext(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(prefs) = data.value.prefs {
                tx.send(Action::Validator(
                    ValidatorAction::UpdateValidatorPrefsNext(account_key, prefs),
                ))
                .boxed()?;
            } else {
                warn!("No validator prefs data found for {}", account_key,);
            }
        }
        Response::ValidatorPayee(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdatePayee(
                account_key,
                data.value.payee,
            )))
            .boxed()?;
        }
        Response::ValidatorNextKeys(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateNextKeys(
                account_key,
                data.value.keys,
            )))
            .boxed()?;
        }
        Response::ValidatorQueuedKeys(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateQueuedKeys(
                account_key,
                data.value.keys,
            )))
            .boxed()?;
        }
        Response::Identity(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(identity) = data.value.identity {
                tx.send(Action::Validator(ValidatorAction::UpdateIdentity(
                    account_key,
                    identity,
                )))
                .boxed()?;
            } else {
                warn!("No identity data found for {}", account_key,);
            }
        }
        Response::TxSubmitted(data) => {
            spawn_process_transaction_progress(runtime, data.value, tx);
        }
        Response::TxValidated => {
            let _ = tx.send(Action::Transaction(TxAction::Message(
                "transaction validated",
            )));
        }
        Response::TxBroadcasted => {
            let _ = tx.send(Action::Transaction(TxAction::Message(
                "transaction broadcasted",
            )));
        }
        Response::TxNoLongerInBestBlock => {
            let _ = tx.send(Action::Transaction(TxAction::Message(
                "transaction no longer in best block",
            )));
        }
        Response::TxInBestBlock(block_hash) => {
            let _ = tx.send(Action::Transaction(TxAction::InBestBlock(block_hash)));
        }
        Response::TxInFinalizedBlock(block_hash) => {
            let _ = tx.send(Action::Transaction(TxAction::InFinalizedBlock(block_hash)));
        }
        Response::TxSuccess => {
            let _ = tx.send(Action::Transaction(TxAction::Success));
        }
        Response::TxError(err) => {
            let _ = tx.send(Action::Transaction(TxAction::Error(err)));
        }
        Response::EventBonded(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::AddAmountToStakeLedger(
                account_key,
                data.value.amount,
            )))
            .boxed()?;
        }
        Response::EventUnbonded(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::SubChunkFromStakeLedger(
                account_key,
                data.value.chunk,
            )))
            .boxed()?;
        }
        Response::EventWithdrawn(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::AddAmountToBalance(
                account_key,
                data.value.amount,
            )))
            .boxed()?;
        }
        Response::SupportedProxy(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            let proxy_key = ProxyKey::new(runtime, data.value.supported_proxy);
            tx.send(Action::Validator(ValidatorAction::AddProxy(
                account_key,
                proxy_key,
            )))
            .boxed()?;
        }
        Response::Balance(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateBalance(
                account_key,
                data.value.balance,
            )))
            .boxed()?;
        } // _ => {
          //     error!("Unhandled response type: {:?}", response);
          // }
    }
    Ok(())
}
