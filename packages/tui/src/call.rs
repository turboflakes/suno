use crate::entry::{AsBytes, ToDescription, ToHex, ToJson, ToMethod, ToPlaceholder};
use log::info;
use serde::Serialize;
use std::str::FromStr;
use subxt::utils::to_hex;
use suno_primitives::staking::{Payee, PayeeError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Call {
    Chill,
    Bond { amount: u128, payee: Payee },
    BondExtra { amount: u128 },
    Unbond { amount: u128 },
    SetPayee { payee: Payee },
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
    #[error("Invalid payee: {0}")]
    InvalidPayee(#[from] PayeeError),
}

impl Call {
    pub fn parse(input: &str, decimals: u32) -> Result<Self, CallError> {
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
                            payee: Payee::None,
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
                                    Ok(Self::Bond { amount, payee })
                                }
                                _ => Err(CallError::UnknownOptional(
                                    "payee <staked|stash|controller|account <address>>".to_string(),
                                )),
                            },
                        }
                    }
                },
                "bond_extra" => match args.split_once(' ') {
                    None => {
                        let amount = parse_standard_unit(args, decimals)?;
                        info!("Bonding extra amount: {} {}  {}", amount, args, decimals);
                        Ok(Self::BondExtra { amount })
                    }
                    _ => Err(CallError::InvalidArgument(input.to_string())),
                },
                "unbond" => match args.split_once(' ') {
                    None => {
                        let amount = parse_standard_unit(args, decimals)?;
                        Ok(Self::Unbond { amount })
                    }
                    _ => Err(CallError::InvalidArgument(input.to_string())),
                },
                "set_payee" => {
                    let payee = Payee::from_str(args)?;
                    Ok(Self::SetPayee { payee })
                }

                // TODO: implement missing calls..
                _ => Err(CallError::InvalidArgument(input.to_string())),
            },
        }
    }
}

// Helper functions
fn parse_standard_unit(value: &str, decimals: u32) -> Result<u128, CallError> {
    match value.split_once('.') {
        None => {
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

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chill => write!(f, "chill"),
            Self::Bond { .. } => write!(f, "bond"),
            Self::BondExtra { .. } => write!(f, "bond_extra"),
            Self::Unbond { .. } => write!(f, "unbond"),
            Self::SetPayee { .. } => write!(f, "set_payee"),
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
            Self::Bond { .. } => "Bond funds".to_string(),
            Self::BondExtra { .. } => "Bond more funds".to_string(),
            Self::Unbond { .. } => "Unbond funds".to_string(),
            Self::SetPayee { .. } => "Set reward destination".to_string(),
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
            Self::BondExtra { .. } => "bond_extra <value-in-standard-units>".to_string(),
            Self::Unbond { .. } => "unbond <value-in-standard-units>".to_string(),
            Self::SetPayee { .. } => {
                "set_payee <staked|stash|controller|account <address>>".to_string()
            }
            Self::ChangeCommission => {
                "change_commission <value-in-percentage> [yes|no]".to_string()
            }
            Self::KickNominators => "kick <address_0, address_1, ...>".to_string(),
            Self::SetSessionKey => "set_keys <session_key>".to_string(),
        }
    }
}

impl ToMethod for Call {
    fn to_method(&self) -> String {
        match self {
            Self::Chill => "chill".to_string(),
            Self::Bond { amount, payee } => format!("bond {amount} payee {payee}"),
            Self::BondExtra { amount } => format!("bond_extra {amount}"),
            Self::Unbond { amount } => format!("unbond {amount}"),
            Self::SetPayee { payee } => format!("set_payee {payee}"),
            _ => "TODO".to_string(),
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
    fn into_bytes(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
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
}
