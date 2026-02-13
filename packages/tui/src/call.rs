use std::str::FromStr;

use crate::entry::{ToDescription, ToHex, ToPlaceholder};
use log::info;
use ratatui::widgets::{Cell, Row};
use subxt::utils::{to_hex, AccountId32};
use suno_primitives::tx::Bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payee {
    Staked,
    Stash,
    Controller,
    Account(AccountId32),
}

/// Convert &str to Payee
impl Payee {
    fn from_str(value: &str) -> Result<Self, CallError> {
        match value.split_once(' ') {
            None => match value {
                "staked" => Ok(Payee::Staked),
                "stash" => Ok(Payee::Stash),
                "controller" => Ok(Payee::Controller),
                _ => Err(CallError::UnknownArgument(
                    "staked|stash|controller|account <address>".to_string(),
                )),
            },
            Some((argument, account)) => match argument {
                "account" => {
                    let acc = AccountId32::from_str(account)
                        .map_err(|_| CallError::InvalidAddress(account.to_string()))?;
                    Ok(Payee::Account(acc))
                }
                _ => Err(CallError::UnknownArgument(
                    "staked|stash|controller|account <address>".to_string(),
                )),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Chill,
    Bond { amount: u128, payee: Option<Payee> },
    Unbond { amount: u128 },
    ChangePayee,
    ChangeCommission,
    KickNominators,
    SetSessionKey,
}

#[derive(thiserror::Error, Debug)]
pub enum CallError {
    #[error("Command must be: {0}")]
    UnknownCommand(String),
    #[error("Optional must be: {0}")]
    UnknownOptional(String),
    #[error("Argument must be one of: {0}")]
    UnknownArgument(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Missing arguments: {0}")]
    MissingArguments(String),
    #[error("No extrinsic provided")]
    MissingExtrinsic,
}

impl Call {
    pub fn parse(input: &str, decimals: u8) -> Result<Self, CallError> {
        match input.split_once(' ') {
            None => match input {
                "chill" => Ok(Self::Chill),
                _ => Err(CallError::MissingExtrinsic),
            },
            Some((extrinsic, args)) => match extrinsic {
                "bond" => match args.split_once(' ') {
                    None => {
                        let amount = parse_standard_unit(args, decimals)?;
                        Ok(Self::Bond {
                            amount,
                            payee: None,
                        })
                    }
                    Some((value, args)) => {
                        let amount = parse_standard_unit(value, decimals)?;
                        match args.split_once(' ') {
                            None => Err(CallError::UnknownOptional(
                                "payee <staked|stash|controller|account <address>>".to_string(),
                            )),
                            Some((payee, args)) => match payee {
                                "payee" => {
                                    let payee = Payee::from_str(args)?;
                                    Ok(Self::Bond {
                                        amount,
                                        payee: Some(payee),
                                    })
                                }
                                _ => Err(CallError::UnknownOptional(
                                    "payee <staked|stash|controller|account <address>>".to_string(),
                                )),
                            },
                        }
                    }
                },
                "unbond" => match args.split_once(' ') {
                    None => {
                        let amount = parse_standard_unit(args, decimals)?;
                        Ok(Self::Unbond { amount })
                    }
                    _ => Err(CallError::InvalidArgument(input.to_string())),
                },
                // TODO: implement missing calls..
                _ => Err(CallError::InvalidArgument(input.to_string())),
            },
        }
    }
}

// Helper functions
fn parse_standard_unit(value: &str, decimals: u8) -> Result<u128, CallError> {
    match value.split_once('.') {
        None => value
            .parse::<u128>()
            .map_err(|_| CallError::InvalidAmount(value.to_string())),
        Some((integer, fractional)) => {
            let integer_part = integer
                .parse::<u128>()
                .map_err(|_| CallError::InvalidAmount(value.to_string()))?;

            if fractional.len() > decimals as usize {
                return Err(CallError::InvalidAmount(format!(
                    "Invalid fractional part (max {})",
                    decimals
                )));
            }

            // Pad with zeros
            let fractional_padded = format!("{:0<width$}", fractional, width = decimals as usize);

            let fractional_part = fractional_padded
                .parse::<u128>()
                .map_err(|e| CallError::InvalidAmount(format!("Invalid fractional part {}", e)))?;

            Ok(integer_part * 10u128.pow(decimals as u32) + fractional_part)
        }
    }
}

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chill => write!(f, "chill"),
            Self::Bond { .. } => write!(f, "bond"),
            Self::Unbond { .. } => write!(f, "unbond"),
            Self::ChangePayee => write!(f, "change_payee"),
            Self::ChangeCommission => write!(f, "change_commission"),
            Self::KickNominators => write!(f, "kick"),
            Self::SetSessionKey => write!(f, "set_keys"),
        }
    }
}

impl ToDescription for Call {
    fn description(&self) -> String {
        match self {
            Self::Chill => "Declare no intention to validate".to_string(),
            Self::Bond { .. } => "Bond more funds".to_string(),
            Self::Unbond { .. } => "Unbond funds".to_string(),
            Self::ChangePayee => "Change reward destination".to_string(),
            Self::ChangeCommission => {
                "Change commission and Allow new nominations by default".to_string()
            }
            Self::KickNominators => "Remove nominators".to_string(),
            Self::SetSessionKey => "Set session keys".to_string(),
        }
    }
}

impl ToPlaceholder for Call {
    fn placeholder(&self) -> String {
        match self {
            Self::Chill => "chill".to_string(),
            Self::Bond { .. } => {
                "bond <value-in-standard-units> [payee <staked|stash|controller|account <address>>]"
                    .to_string()
            }
            Self::Unbond { .. } => "unbond <value-in-standard-units>".to_string(),
            Self::ChangePayee => {
                "change_payee <staked|stash|controller|account <address>>".to_string()
            }
            Self::ChangeCommission => {
                "change_commission <value-in-percentage> [yes|no]".to_string()
            }
            Self::KickNominators => "kick <address_0, address_1, ...>".to_string(),
            Self::SetSessionKey => "set_keys <session_key>".to_string(),
        }
    }
}

impl ToHex for Call {
    // TODO: Implement to_hex method for Call enum
    fn to_hex(&self) -> String {
        match self {
            // Self::Chill(bytes) => to_hex(bytes),
            Self::Chill => "Chill".to_string(),
            Self::Bond { .. } => "Bond more funds".to_string(),
            Self::Unbond { .. } => "Unbond funds".to_string(),
            Self::ChangePayee => "Change reward destination".to_string(),
            Self::ChangeCommission => "Change commission".to_string(),
            Self::KickNominators => "Kick nominators".to_string(),
            Self::SetSessionKey => "Change session keys".to_string(),
        }
    }
}
