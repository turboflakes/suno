use crate::entry::{AsBytes, ToDescription, ToHex, ToJson, ToMethod, ToPlaceholder};
use crate::session::{Keys, KeysError};
use crate::staking::{Payee, PayeeError};
use serde::Serialize;
use sp_arithmetic::Perbill;
use std::str::FromStr;
use subxt::utils::to_hex;

type Amount = u128;
type Description = String;
type Max = Option<(Amount, Description)>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Call {
    Bond {
        amount: u128,
        payee: Payee,
        max: Max,
    },
    BondExtra {
        amount: u128,
        max: Max,
    },
    Unbond {
        amount: u128,
        max: Max,
    },
    Rebond {
        amount: u128,
        max: Max,
    },
    WithdrawUnbonded {
        max: Max,
    },
    SetPayee {
        payee: Payee,
    },
    Validate {
        commission: Perbill,
        blocked: bool,
    },
    Chill,
    SetSessionKeys {
        keys: Keys,
    },
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
    #[error("Invalid percentage: {0}")]
    InvalidPercentage(String),
    #[error("Invalid percentage {0}%, value must be between 0 and 100")]
    InvalidPercentageRange(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("No argument provided")]
    MissingArgument,
    #[error("No extrinsic provided")]
    MissingExtrinsic,
    #[error("Invalid payee: {0}")]
    InvalidPayee(#[from] PayeeError),
    #[error("Invalid keys: {0}")]
    InvalidKeys(#[from] KeysError),
}

impl Call {
    pub fn parse(input: &str, decimals: u32) -> Result<Self, CallError> {
        match input.split_once(' ') {
            None => match input {
                "chill" => Ok(Self::Chill),
                "withdraw_unbonded" => Ok(Self::WithdrawUnbonded { max: None }),
                _ => Err(CallError::MissingExtrinsic),
            },
            Some((extrinsic, args)) => match extrinsic {
                "bond" => match args.split_once(' ') {
                    None => {
                        let amount = parse_standard_unit(args, decimals)?;
                        Ok(Self::Bond {
                            amount,
                            payee: Payee::None,
                            max: None,
                        })
                    }
                    Some((value, args)) => {
                        let amount = parse_standard_unit(value, decimals)?;
                        match args.split_once(' ') {
                            None => Err(CallError::UnknownOptional(
                                "payee <staked|stash|account <address>>".to_string(),
                            )),
                            Some(("payee", args)) => {
                                let payee = Payee::from_str(args)?;
                                Ok(Self::Bond {
                                    amount,
                                    payee,
                                    max: None,
                                })
                            }
                            Some((_other, _)) => Err(CallError::UnknownOptional(
                                "payee <staked|stash|account <address>>".to_string(),
                            )),
                        }
                    }
                },
                "bond_extra" => match args.split_once(' ') {
                    None => {
                        let amount = parse_standard_unit(args, decimals)?;
                        Ok(Self::BondExtra { amount, max: None })
                    }
                    _ => Err(CallError::InvalidArgument(input.to_string())),
                },
                "unbond" => match args.split_once(' ') {
                    None => {
                        let amount = parse_standard_unit(args, decimals)?;
                        Ok(Self::Unbond { amount, max: None })
                    }
                    _ => Err(CallError::InvalidArgument(input.to_string())),
                },
                "rebond" => match args.split_once(' ') {
                    None => {
                        let amount = parse_standard_unit(args, decimals)?;
                        Ok(Self::Rebond { amount, max: None })
                    }
                    _ => Err(CallError::InvalidArgument(input.to_string())),
                },
                "set_payee" => {
                    let payee = Payee::from_str(args)?;
                    Ok(Self::SetPayee { payee })
                }
                "set_keys" => {
                    let keys = Keys::from_str(args)?;
                    Ok(Self::SetSessionKeys { keys })
                }
                "validate" => match args.split_once(' ') {
                    None => {
                        let commission = parse_percentage(args)?;
                        Ok(Self::Validate {
                            commission,
                            blocked: false,
                        })
                    }
                    Some((value, args)) => {
                        let commission = parse_percentage(value)?;
                        match args.split_once(' ') {
                            None => Err(CallError::UnknownOptional("blocked <yes|no>".to_string())),
                            Some((blocked, args)) => match blocked {
                                "blocked" => {
                                    let blocked = parse_boolean(args)?;
                                    Ok(Self::Validate {
                                        commission,
                                        blocked,
                                    })
                                }
                                _ => {
                                    Err(CallError::UnknownOptional("blocked <yes|no>".to_string()))
                                }
                            },
                        }
                    }
                },
                _ => Err(CallError::InvalidArgument(input.to_string())),
            },
        }
    }
}

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bond { .. } => write!(f, "bond"),
            Self::BondExtra { .. } => write!(f, "bond_extra"),
            Self::Unbond { .. } => write!(f, "unbond"),
            Self::Rebond { .. } => write!(f, "rebond"),
            Self::WithdrawUnbonded { .. } => write!(f, "withdraw_unbonded"),
            Self::SetPayee { .. } => write!(f, "set_payee"),
            Self::Validate { .. } => write!(f, "validate"),
            Self::Chill => write!(f, "chill"),
            Self::SetSessionKeys { .. } => write!(f, "set_keys"),
        }
    }
}

impl ToDescription for Call {
    fn description(&self) -> String {
        match self {
            Self::Bond { max, .. } => format!(
                "Bond up to {} from your free balance",
                max.as_ref()
                    .map(|(_, description)| description.to_string())
                    .unwrap_or_default()
            ),
            Self::BondExtra { max, .. } => format!(
                "Bond extra funds, up to {} from your free balance",
                max.as_ref()
                    .map(|(_, description)| description.to_string())
                    .unwrap_or_default()
            )
            .to_string(),
            Self::Unbond { max, .. } => format!(
                "Unbond up to {} that is bonded",
                max.as_ref()
                    .map(|(_, description)| description.to_string())
                    .unwrap_or_default()
            ),
            Self::Rebond { max, .. } => format!(
                "Rebond up to {} that is currently unlocking",
                max.as_ref()
                    .map(|(_, description)| description.to_string())
                    .unwrap_or_default()
            ),
            Self::WithdrawUnbonded { max } => {
                format!(
                    "Withdraw the {} that are fully unlocked",
                    max.as_ref()
                        .map(|(_, description)| description.to_string())
                        .unwrap_or_default()
                )
            }
            Self::SetPayee { .. } => "Set reward destination".to_string(),
            Self::Validate { .. } => {
                "Validate/Change commission or enable/disable nominations".to_string()
            }
            Self::Chill => "Declare no intention to validate".to_string(),
            Self::SetSessionKeys { .. } => {
                "Set session keys from the output of 'author_rotateKeys' call".to_string()
            }
        }
    }
}

impl ToPlaceholder for Call {
    fn placeholder(&self) -> String {
        match self {
            Self::Bond { .. } => {
                "bond <value-in-standard-units> [payee <staked|stash|account <address>>]"
                    .to_string()
            }
            Self::BondExtra { .. } => "bond_extra <value-in-standard-units>".to_string(),
            Self::Unbond { .. } => "unbond <value-in-standard-units>".to_string(),
            Self::Rebond { .. } => "rebond <value-in-standard-units>".to_string(),
            Self::WithdrawUnbonded { .. } => "withdraw_unbonded".to_string(),
            Self::SetPayee { .. } => "set_payee <staked|stash|account <address>>".to_string(),
            Self::Validate { .. } => {
                "validate <value-in-percentage> [blocked <yes|no>]".to_string()
            }
            Self::Chill => "chill".to_string(),
            Self::SetSessionKeys { .. } => {
                "set_keys <hex-session-keys-from-author-rotate-keys>".to_string()
            }
        }
    }
}

impl ToMethod for Call {
    fn to_method(&self) -> String {
        match self {
            Self::Bond { amount, payee, .. } => format!("bond {amount} payee {payee}"),
            Self::BondExtra { amount, .. } => format!("bond_extra {amount}"),
            Self::Unbond { amount, .. } => format!("unbond {amount}"),
            Self::Rebond { amount, .. } => format!("rebond {amount}"),
            Self::WithdrawUnbonded { .. } => "withdraw_unbonded".to_string(),
            Self::SetPayee { payee } => format!("set_payee {payee}"),
            Self::Validate {
                commission,
                blocked,
            } => format!("validate {} blocked {blocked}", commission.deconstruct()),
            Self::Chill => "chill".to_string(),
            Self::SetSessionKeys { keys } => format!("set_keys {keys}"),
        }
    }
}

impl ToJson for Call {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

impl ToHex for Call {
    fn to_hex(&self) -> String {
        to_hex(self.to_string().as_bytes())
    }
}

impl AsBytes for Call {
    fn as_bytes(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

// Helper functions
fn parse_standard_unit(value: &str, decimals: u32) -> Result<u128, CallError> {
    match value.split_once('.') {
        None => {
            if value.is_empty() {
                return Err(CallError::MissingArgument);
            }
            let value = value
                .parse::<u128>()
                .map_err(|_| CallError::InvalidAmount(value.to_string()))?;
            Ok(value * 10u128.pow(decimals))
        }
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

            Ok(integer_part * 10u128.pow(decimals) + fractional_part)
        }
    }
}

fn parse_percentage(value: &str) -> Result<Perbill, CallError> {
    let percent_value = value
        .trim()
        .parse::<f64>()
        .map_err(|_| CallError::InvalidPercentage(value.to_string()))?;

    // Validate range
    if !(0.0..=100.0).contains(&percent_value) {
        return Err(CallError::InvalidPercentageRange(percent_value.to_string()));
    }

    // Convert percentage to fraction of 1 billion (Perbill's base)
    // 22.5% = 0.225 = 225_000_000 / 1_000_000_000
    let parts = (percent_value * 10_000_000.0).round() as u32;

    Ok(Perbill::from_parts(parts))
}

fn parse_boolean(value: &str) -> Result<bool, CallError> {
    match value.trim().to_lowercase().as_str() {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(CallError::InvalidArgument(
            "expected 'yes' or 'no'".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_unit_integer_only() {
        // Test whole numbers without decimal points
        assert_eq!(parse_standard_unit("1", 10).unwrap(), 10_000_000_000);
        assert_eq!(parse_standard_unit("100", 10).unwrap(), 1_000_000_000_000);
        assert_eq!(parse_standard_unit("0", 10).unwrap(), 0);
        assert_eq!(parse_standard_unit("999", 12).unwrap(), 999_000_000_000_000);
        assert_eq!(parse_standard_unit("100", 0).unwrap(), 100);
    }

    #[test]
    fn test_parse_standard_unit_with_decimals() {
        // Test with decimal points - standard cases
        assert_eq!(parse_standard_unit("1.5", 10).unwrap(), 15_000_000_000);
        assert_eq!(parse_standard_unit("0.1", 10).unwrap(), 1_000_000_000);
        assert_eq!(parse_standard_unit("10.25", 10).unwrap(), 102_500_000_000);
        assert_eq!(
            parse_standard_unit("100.123", 10).unwrap(),
            1_001_230_000_000
        );
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("0").unwrap().deconstruct(), 0);
        assert_eq!(parse_percentage("1").unwrap().deconstruct(), 10_000_000);
        assert_eq!(parse_percentage("22.5").unwrap().deconstruct(), 225_000_000);
        assert_eq!(parse_percentage("50").unwrap().deconstruct(), 500_000_000);
        assert_eq!(
            parse_percentage("100").unwrap().deconstruct(),
            1_000_000_000
        );
    }
}
