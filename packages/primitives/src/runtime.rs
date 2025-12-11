use serde::{Deserialize, Serialize};

pub type ChainPrefix = u16;
pub type ChainTokenSymbol = String;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SupportedRuntime {
    Local,
    Polkadot,
    Kusama,
    Paseo,
    Westend,
    #[serde(rename = "asset_hub_polkadot")]
    AssetHubPolkadot,
    #[serde(rename = "bridge_hub_polkadot")]
    BridgeHubPolkadot,
    #[serde(rename = "people_polkadot")]
    PeoplePolkadot,
    #[serde(rename = "asset_hub_kusama")]
    AssetHubKusama,
    #[serde(rename = "bridge_hub_kusama")]
    BridgeHubKusama,
    #[serde(rename = "people_kusama")]
    PeopleKusama,
    #[serde(rename = "asset_hub_paseo")]
    AssetHubPaseo,
    #[serde(rename = "bridge_hub_paseo")]
    BridgeHubPaseo,
    #[serde(rename = "people_paseo")]
    PeoplePaseo,
    #[serde(rename = "asset_hub_westend")]
    AssetHubWestend,
    #[serde(rename = "bridge_hub_westend")]
    BridgeHubWestend,
    #[serde(rename = "people_westend")]
    PeopleWestend,
}

impl SupportedRuntime {
    pub fn chain_prefix(&self) -> ChainPrefix {
        match &self {
            Self::Polkadot
            | Self::AssetHubPolkadot
            | Self::BridgeHubPolkadot
            | Self::PeoplePolkadot => 0,
            Self::Kusama | Self::AssetHubKusama | Self::BridgeHubKusama | Self::PeopleKusama => 2,
            Self::Local
            | Self::Paseo
            | Self::AssetHubPaseo
            | Self::BridgeHubPaseo
            | Self::PeoplePaseo
            | Self::Westend
            | Self::AssetHubWestend
            | Self::BridgeHubWestend
            | Self::PeopleWestend => 42,
        }
    }

    pub fn asset_hub_runtime(&self) -> Self {
        match &self {
            Self::Polkadot => Self::AssetHubPolkadot,
            Self::Kusama => Self::AssetHubKusama,
            Self::Paseo => Self::AssetHubPaseo,
            Self::Westend => Self::AssetHubWestend,
            _ => unimplemented!("AssetHub runtime not supported"),
        }
    }

    pub fn bridge_hub_runtime(&self) -> Self {
        match &self {
            Self::Polkadot => Self::BridgeHubPolkadot,
            Self::Kusama => Self::BridgeHubKusama,
            Self::Paseo => Self::BridgeHubPaseo,
            Self::Westend => Self::BridgeHubWestend,
            _ => unimplemented!("BridgeHub runtime not supported"),
        }
    }

    pub fn people_runtime(&self) -> Self {
        match &self {
            Self::Polkadot => Self::PeoplePolkadot,
            Self::Kusama => Self::PeopleKusama,
            Self::Paseo => Self::PeoplePaseo,
            Self::Westend => Self::PeopleWestend,
            _ => unimplemented!("People runtime not supported"),
        }
    }
}

impl std::fmt::Display for SupportedRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Local => write!(f, "Local"),
            Self::Polkadot => write!(f, "Polkadot"),
            Self::Kusama => write!(f, "Kusama"),
            Self::Paseo => write!(f, "Paseo"),
            Self::Westend => write!(f, "Westend"),
            Self::AssetHubPolkadot => write!(f, "AssetHub Polkadot"),
            Self::BridgeHubPolkadot => write!(f, "BridgeHub Polkadot"),
            Self::PeoplePolkadot => write!(f, "People Polkadot"),
            Self::AssetHubKusama => write!(f, "AssetHub Kusama"),
            Self::BridgeHubKusama => write!(f, "BridgeHub Kusama"),
            Self::PeopleKusama => write!(f, "People Kusama"),
            Self::AssetHubPaseo => write!(f, "AssetHub Paseo"),
            Self::BridgeHubPaseo => write!(f, "BridgeHub Paseo"),
            Self::PeoplePaseo => write!(f, "People Paseo"),
            Self::AssetHubWestend => write!(f, "AssetHub Westend"),
            Self::BridgeHubWestend => write!(f, "BridgeHub Westend"),
            Self::PeopleWestend => write!(f, "People Westend"),
        }
    }
}
