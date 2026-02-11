use crate::entry::{Command, Entry, ToDescription, ToHex, ToPlaceholder};
use crate::widgets::popup::Mode;
use ratatui::{
    layout::Alignment,
    text::Line,
    widgets::{Cell, Row},
};
use subxt::utils::to_hex;
use suno_primitives::tx::Bytes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Chill(Bytes),
    Bond,
    Unbond,
    ChangePayee,
    ChangeCommission,
    KickNominators,
    SetSessionKey,
}

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chill(_) => write!(f, "chill"),
            Self::Bond => write!(f, "bond"),
            Self::Unbond => write!(f, "unbond"),
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
            Self::Chill(_) => "Declare no intention to validate".to_string(),
            Self::Bond => "Bond more funds".to_string(),
            Self::Unbond => "Unbond funds".to_string(),
            Self::ChangePayee => "Change reward destination".to_string(),
            Self::ChangeCommission => "Change commission and allow new nominations".to_string(),
            Self::KickNominators => "Remove nominators".to_string(),
            Self::SetSessionKey => "Set/Change session keys".to_string(),
        }
    }
}

impl ToPlaceholder for Call {
    fn placeholder(&self) -> String {
        match self {
            Self::Chill(_) => "chill".to_string(),
            Self::Bond => {
                "bond <value> [payee <staked|stash|controller|account <address>>]".to_string()
            }
            Self::Unbond => "unbond <value>".to_string(),
            Self::ChangePayee => {
                "change_payee <staked|stash|controller|account <address>>".to_string()
            }
            Self::ChangeCommission => "change_commission <value> [yes|no]".to_string(),
            Self::KickNominators => "kick <address_0, address_1, ...>".to_string(),
            Self::SetSessionKey => "set_keys <session_key>".to_string(),
        }
    }
}

impl ToHex for Call {
    fn to_hex(&self) -> String {
        match self {
            Self::Chill(bytes) => to_hex(bytes),
            Self::Bond => "Bond more funds".to_string(),
            Self::Unbond => "Unbond funds".to_string(),
            Self::ChangePayee => "Change reward destination".to_string(),
            Self::ChangeCommission => "Change commission".to_string(),
            Self::KickNominators => "Kick nominators".to_string(),
            Self::SetSessionKey => "Change session keys".to_string(),
        }
    }
}
