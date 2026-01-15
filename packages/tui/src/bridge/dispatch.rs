use log::{error, warn};
use suno_actions::{Action, ChainAction, ValidatorAction};
use suno_config::SupportedRuntime;
use suno_error::Error;
use suno_primitives::{AccountKey, Response};
use tokio::sync::mpsc::UnboundedSender;

pub fn dispatch_response_action(
    response: Response,
    runtime: &SupportedRuntime,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    match response {
        Response::Era(data) => {
            tx.send(Action::Chain(ChainAction::UpdateEra(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::Epoch(data) => {
            tx.send(Action::Chain(ChainAction::UpdateEpoch(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::TotalStaked(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalStaked(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::ActiveValidators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateActiveValidators(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::ActiveNominators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateActiveNominators(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::AuthorityStatus(data) => {
            let account_key = AccountKey::from_bytes(runtime.clone(), data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateStatus(
                account_key,
                data.value.status,
            )))?;
        }
        Response::AuthorityEraPoints(data) => {
            let account_key = AccountKey::from_bytes(runtime.clone(), data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateEraPoints(
                account_key,
                data.value.points,
            )))?;
        }
        Response::StakeLedger(data) => {
            let account_key = AccountKey::from_bytes(runtime.clone(), data.value.account);
            if let Some(ledger) = data.value.ledger {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeLedger(
                    account_key,
                    ledger,
                )))?;
            } else {
                warn!("No stake ledger data found for {}", account_key.to_string(),);
            }
        }
        Response::TotalValidators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalValidators(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::TotalNominators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalNominators(
                runtime.clone(),
                data.value,
            )))?;
        }
        _ => {
            error!("Unhandled response type: {:?}", response);
        }
    }
    Ok(())
}
