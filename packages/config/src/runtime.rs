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

pub type Runtime = SupportedRuntime;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
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
    pub fn relay_chain(&self) -> Self {
        match &self {
            Self::Local => Self::Local,
            Self::Polkadot
            | Self::AssetHubPolkadot
            | Self::BridgeHubPolkadot
            | Self::PeoplePolkadot => Self::Polkadot,
            Self::Kusama | Self::AssetHubKusama | Self::BridgeHubKusama | Self::PeopleKusama => {
                Self::Kusama
            }
            Self::Paseo | Self::AssetHubPaseo | Self::BridgeHubPaseo | Self::PeoplePaseo => {
                Self::Paseo
            }
            Self::Westend
            | Self::AssetHubWestend
            | Self::BridgeHubWestend
            | Self::PeopleWestend => Self::Westend,
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

    pub fn account_format(&self) -> u16 {
        match &self {
            Self::Polkadot => get_ss58_format(POLKADOT_SPEC),
            Self::Kusama => get_ss58_format(KUSAMA_SPEC),
            Self::Westend => get_ss58_format(WESTEND_SPEC),
            Self::Paseo => get_ss58_format(PASEO_SPEC),
            Self::PeoplePolkadot => get_ss58_format(PEOPLE_POLKADOT_SPEC),
            Self::PeopleKusama => get_ss58_format(PEOPLE_KUSAMA_SPEC),
            Self::PeopleWestend => get_ss58_format(PEOPLE_WESTEND_SPEC),
            Self::PeoplePaseo => get_ss58_format(PEOPLE_PASEO_SPEC),
            Self::AssetHubPolkadot => get_ss58_format(ASSET_HUB_POLKADOT_SPEC),
            Self::AssetHubKusama => get_ss58_format(ASSET_HUB_KUSAMA_SPEC),
            Self::AssetHubWestend => get_ss58_format(ASSET_HUB_WESTEND_SPEC),
            Self::AssetHubPaseo => get_ss58_format(ASSET_HUB_PASEO_SPEC),
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn token_symbol(&self) -> String {
        match &self {
            Self::Polkadot => get_symbol(POLKADOT_SPEC),
            Self::Kusama => get_symbol(KUSAMA_SPEC),
            Self::Westend => get_symbol(WESTEND_SPEC),
            Self::Paseo => get_symbol(PASEO_SPEC),
            Self::PeoplePolkadot => get_symbol(PEOPLE_POLKADOT_SPEC),
            Self::PeopleKusama => get_symbol(PEOPLE_KUSAMA_SPEC),
            Self::PeopleWestend => get_symbol(PEOPLE_WESTEND_SPEC),
            Self::PeoplePaseo => get_symbol(PEOPLE_PASEO_SPEC),
            Self::AssetHubPolkadot => get_symbol(ASSET_HUB_POLKADOT_SPEC),
            Self::AssetHubKusama => get_symbol(ASSET_HUB_KUSAMA_SPEC),
            Self::AssetHubWestend => get_symbol(ASSET_HUB_WESTEND_SPEC),
            Self::AssetHubPaseo => get_symbol(ASSET_HUB_PASEO_SPEC),
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn token_decimals(&self) -> u32 {
        match &self {
            Self::Polkadot => get_decimals(POLKADOT_SPEC),
            Self::Kusama => get_decimals(KUSAMA_SPEC),
            Self::Westend => get_decimals(WESTEND_SPEC),
            Self::Paseo => get_decimals(PASEO_SPEC),
            Self::PeoplePolkadot => get_decimals(PEOPLE_POLKADOT_SPEC),
            Self::PeopleKusama => get_decimals(PEOPLE_KUSAMA_SPEC),
            Self::PeopleWestend => get_decimals(PEOPLE_WESTEND_SPEC),
            Self::PeoplePaseo => get_decimals(PEOPLE_PASEO_SPEC),
            Self::AssetHubPolkadot => get_decimals(ASSET_HUB_POLKADOT_SPEC),
            Self::AssetHubKusama => get_decimals(ASSET_HUB_KUSAMA_SPEC),
            Self::AssetHubWestend => get_decimals(ASSET_HUB_WESTEND_SPEC),
            Self::AssetHubPaseo => get_decimals(ASSET_HUB_PASEO_SPEC),
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn is_relay_chain(&self) -> bool {
        match &self {
            Self::Local | Self::Polkadot | Self::Kusama | Self::Paseo | Self::Westend => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Polkadot => "Polkadot",
            Self::Kusama => "Kusama",
            Self::Westend => "Westend",
            Self::Paseo => "Paseo",
            Self::AssetHubPolkadot => "AH Polkadot",
            Self::BridgeHubPolkadot => "BH Polkadot",
            Self::PeoplePolkadot => "People Polkadot",
            Self::AssetHubKusama => "AH Kusama",
            Self::BridgeHubKusama => "BH Kusama",
            Self::PeopleKusama => "People Kusama",
            Self::AssetHubPaseo => "AH Paseo",
            Self::BridgeHubPaseo => "BH Paseo",
            Self::PeoplePaseo => "People Paseo",
            Self::AssetHubWestend => "AH Westend",
            Self::BridgeHubWestend => "BH Westend",
            Self::PeopleWestend => "People Westend",
        }
    }
    pub fn as_str_long(&self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Polkadot => "Polkadot",
            Self::Kusama => "Kusama",
            Self::Westend => "Westend",
            Self::Paseo => "Paseo",
            Self::AssetHubPolkadot => "Asset Hub Polkadot",
            Self::BridgeHubPolkadot => "Bridge Hub Polkadot",
            Self::PeoplePolkadot => "People Polkadot",
            Self::AssetHubKusama => "Asset Hub Kusama",
            Self::BridgeHubKusama => "Bridge Hub Kusama",
            Self::PeopleKusama => "People Kusama",
            Self::AssetHubPaseo => "Asset Hub Paseo",
            Self::BridgeHubPaseo => "Bridge Hub Paseo",
            Self::PeoplePaseo => "People Paseo",
            Self::AssetHubWestend => "Asset Hub Westend",
            Self::BridgeHubWestend => "Bridge Hub Westend",
            Self::PeopleWestend => "People Westend",
        }
    }
}

impl std::fmt::Display for SupportedRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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

fn get_ss58_format(chain_specs: &str) -> u16 {
    let spec: Result<Value> = serde_json::from_str(chain_specs);
    match spec {
        Ok(json) => {
            let value = json["properties"]["ss58Format"]
                .as_u64()
                .expect("chain spec does not contain ss58Format");
            value as u16
        }
        Err(err) => panic!("Failed to parse JSON: {}", err),
    }
}

fn get_symbol(chain_specs: &str) -> String {
    let spec: Result<Value> = serde_json::from_str(chain_specs);
    match spec {
        Ok(json) => {
            let value = json["properties"]["tokenSymbol"]
                .as_str()
                .expect("chain spec does not contain tokenSymbol");
            value.to_string()
        }
        Err(err) => panic!("Failed to parse JSON: {}", err),
    }
}

fn get_decimals(chain_specs: &str) -> u32 {
    let spec: Result<Value> = serde_json::from_str(chain_specs);
    match spec {
        Ok(json) => {
            let value = json["properties"]["tokenDecimals"]
                .as_u64()
                .expect("chain spec does not contain tokenDecimals");
            value as u32
        }
        Err(err) => panic!("Failed to parse JSON: {}", err),
    }
}
