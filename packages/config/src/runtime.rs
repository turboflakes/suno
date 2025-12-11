use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::str::FromStr;
use subxt::utils::H256;

pub const POLKADOT_SPEC: &str = include_str!("../chain-specs/polkadot.json");
pub const ASSET_HUB_POLKADOT_SPEC: &str = include_str!("../chain-specs/asset-hub-polkadot.json");
pub const PEOPLE_POLKADOT_SPEC: &str = include_str!("../chain-specs/people-polkadot.json");

pub const KUSAMA_SPEC: &str = include_str!("../chain-specs/kusama.json");
pub const ASSET_HUB_KUSAMA_SPEC: &str = include_str!("../chain-specs/asset-hub-kusama.json");
pub const PEOPLE_KUSAMA_SPEC: &str = include_str!("../chain-specs/people-kusama.json");

pub const PASEO_SPEC: &str = include_str!("../chain-specs/paseo.json");
pub const ASSET_HUB_PASEO_SPEC: &str = include_str!("../chain-specs/asset-hub-paseo.json");
pub const PEOPLE_PASEO_SPEC: &str = include_str!("../chain-specs/people-paseo.json");

pub const WESTEND_SPEC: &str = include_str!("../chain-specs/westend.json");
pub const ASSET_HUB_WESTEND_SPEC: &str = include_str!("../chain-specs/asset-hub-westend.json");
pub const PEOPLE_WESTEND_SPEC: &str = include_str!("../chain-specs/people-westend.json");

pub type ChainPrefix = u16;

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

    pub fn chain_specs(&self) -> &str {
        match &self {
            Self::Polkadot => POLKADOT_SPEC,
            Self::Kusama => KUSAMA_SPEC,
            Self::Westend => WESTEND_SPEC,
            Self::Paseo => PASEO_SPEC,
            Self::PeoplePolkadot => PEOPLE_POLKADOT_SPEC,
            Self::PeopleKusama => PEOPLE_KUSAMA_SPEC,
            Self::PeopleWestend => PEOPLE_WESTEND_SPEC,
            Self::PeoplePaseo => PEOPLE_PASEO_SPEC,
            Self::AssetHubPolkadot => ASSET_HUB_POLKADOT_SPEC,
            Self::AssetHubKusama => ASSET_HUB_KUSAMA_SPEC,
            Self::AssetHubWestend => ASSET_HUB_WESTEND_SPEC,
            Self::AssetHubPaseo => ASSET_HUB_PASEO_SPEC,
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn chain_state_root_hash(&self) -> H256 {
        match &self {
            Self::Polkadot => get_state_root_hash(POLKADOT_SPEC),
            Self::Kusama => get_state_root_hash(KUSAMA_SPEC),
            Self::Westend => get_state_root_hash(WESTEND_SPEC),
            Self::Paseo => get_state_root_hash(PASEO_SPEC),
            Self::PeoplePolkadot => get_state_root_hash(PEOPLE_POLKADOT_SPEC),
            Self::PeopleKusama => get_state_root_hash(PEOPLE_KUSAMA_SPEC),
            Self::PeopleWestend => get_state_root_hash(PEOPLE_WESTEND_SPEC),
            Self::PeoplePaseo => get_state_root_hash(PEOPLE_PASEO_SPEC),
            Self::AssetHubPolkadot => get_state_root_hash(ASSET_HUB_POLKADOT_SPEC),
            Self::AssetHubKusama => get_state_root_hash(ASSET_HUB_KUSAMA_SPEC),
            Self::AssetHubWestend => get_state_root_hash(ASSET_HUB_WESTEND_SPEC),
            Self::AssetHubPaseo => get_state_root_hash(ASSET_HUB_PASEO_SPEC),
            _ => panic!("Unsupported chain"),
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

fn get_state_root_hash(chain_specs: &str) -> H256 {
    let spec: Result<Value> = serde_json::from_str(chain_specs);
    match spec {
        Ok(json) => {
            let state_root = json["genesis"]["stateRootHash"]
                .as_str()
                .expect("chain spec does not contain state root hash");
            H256::from_str(state_root).expect("invalid state root hash")
        }
        Err(err) => panic!("Failed to parse JSON: {}", err),
    }
}
