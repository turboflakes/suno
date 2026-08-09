use crate::config::CONFIG;
use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use subxt::utils::{AccountId32, H256};
use tracing::info;

#[cfg(feature = "polkadot")]
pub const POLKADOT_SPEC: &str = include_str!("../chain-specs/polkadot.json");
#[cfg(feature = "polkadot")]
pub const ASSET_HUB_POLKADOT_SPEC: &str = include_str!("../chain-specs/asset-hub-polkadot.json");
#[cfg(feature = "polkadot")]
pub const PEOPLE_POLKADOT_SPEC: &str = include_str!("../chain-specs/people-polkadot.json");

#[cfg(feature = "kusama")]
pub const KUSAMA_SPEC: &str = include_str!("../chain-specs/kusama.json");
#[cfg(feature = "kusama")]
pub const ASSET_HUB_KUSAMA_SPEC: &str = include_str!("../chain-specs/asset-hub-kusama.json");
#[cfg(feature = "kusama")]
pub const PEOPLE_KUSAMA_SPEC: &str = include_str!("../chain-specs/people-kusama.json");

#[cfg(feature = "paseo")]
pub const PASEO_SPEC: &str = include_str!("../chain-specs/paseo.json");
#[cfg(feature = "paseo")]
pub const ASSET_HUB_PASEO_SPEC: &str = include_str!("../chain-specs/asset-hub-paseo.json");
#[cfg(feature = "paseo")]
pub const PEOPLE_PASEO_SPEC: &str = include_str!("../chain-specs/people-paseo.json");

#[cfg(feature = "westend")]
pub const WESTEND_SPEC: &str = include_str!("../chain-specs/westend.json");
#[cfg(feature = "westend")]
pub const ASSET_HUB_WESTEND_SPEC: &str = include_str!("../chain-specs/asset-hub-westend.json");
#[cfg(feature = "westend")]
pub const PEOPLE_WESTEND_SPEC: &str = include_str!("../chain-specs/people-westend.json");

pub type Runtime = SupportedRuntime;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SupportedRuntime {
    Local,
    #[cfg(feature = "polkadot")]
    Polkadot,
    #[cfg(feature = "kusama")]
    Kusama,
    #[cfg(feature = "paseo")]
    Paseo,
    #[cfg(feature = "westend")]
    Westend,
    #[cfg(feature = "polkadot")]
    #[serde(rename = "asset_hub_polkadot")]
    AssetHubPolkadot,
    #[cfg(feature = "polkadot")]
    #[serde(rename = "bridge_hub_polkadot")]
    BridgeHubPolkadot,
    #[cfg(feature = "polkadot")]
    #[serde(rename = "people_polkadot")]
    PeoplePolkadot,
    #[cfg(feature = "kusama")]
    #[serde(rename = "asset_hub_kusama")]
    AssetHubKusama,
    #[cfg(feature = "kusama")]
    #[serde(rename = "bridge_hub_kusama")]
    BridgeHubKusama,
    #[cfg(feature = "kusama")]
    #[serde(rename = "people_kusama")]
    PeopleKusama,
    #[cfg(feature = "paseo")]
    #[serde(rename = "asset_hub_paseo")]
    AssetHubPaseo,
    #[cfg(feature = "paseo")]
    #[serde(rename = "bridge_hub_paseo")]
    BridgeHubPaseo,
    #[cfg(feature = "paseo")]
    #[serde(rename = "people_paseo")]
    PeoplePaseo,
    #[cfg(feature = "westend")]
    #[serde(rename = "asset_hub_westend")]
    AssetHubWestend,
    #[cfg(feature = "westend")]
    #[serde(rename = "bridge_hub_westend")]
    BridgeHubWestend,
    #[cfg(feature = "westend")]
    #[serde(rename = "people_westend")]
    PeopleWestend,
}

impl SupportedRuntime {
    pub fn relay_chain(&self) -> Self {
        match &self {
            Self::Local => Self::Local,
            #[cfg(feature = "polkadot")]
            Self::Polkadot
            | Self::AssetHubPolkadot
            | Self::BridgeHubPolkadot
            | Self::PeoplePolkadot => Self::Polkadot,
            #[cfg(feature = "kusama")]
            Self::Kusama | Self::AssetHubKusama | Self::BridgeHubKusama | Self::PeopleKusama => {
                Self::Kusama
            }
            #[cfg(feature = "paseo")]
            Self::Paseo | Self::AssetHubPaseo | Self::BridgeHubPaseo | Self::PeoplePaseo => {
                Self::Paseo
            }
            #[cfg(feature = "westend")]
            Self::Westend
            | Self::AssetHubWestend
            | Self::BridgeHubWestend
            | Self::PeopleWestend => Self::Westend,
        }
    }

    pub fn asset_hub_runtime(&self) -> Self {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => Self::AssetHubPolkadot,
            #[cfg(feature = "kusama")]
            Self::Kusama => Self::AssetHubKusama,
            #[cfg(feature = "paseo")]
            Self::Paseo => Self::AssetHubPaseo,
            #[cfg(feature = "westend")]
            Self::Westend => Self::AssetHubWestend,
            _ => unimplemented!("AssetHub runtime not supported"),
        }
    }

    pub fn bridge_hub_runtime(&self) -> Self {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => Self::BridgeHubPolkadot,
            #[cfg(feature = "kusama")]
            Self::Kusama => Self::BridgeHubKusama,
            #[cfg(feature = "paseo")]
            Self::Paseo => Self::BridgeHubPaseo,
            #[cfg(feature = "westend")]
            Self::Westend => Self::BridgeHubWestend,
            _ => unimplemented!("BridgeHub runtime not supported"),
        }
    }

    pub fn people_runtime(&self) -> Self {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => Self::PeoplePolkadot,
            #[cfg(feature = "kusama")]
            Self::Kusama => Self::PeopleKusama,
            #[cfg(feature = "paseo")]
            Self::Paseo => Self::PeoplePaseo,
            #[cfg(feature = "westend")]
            Self::Westend => Self::PeopleWestend,
            _ => unimplemented!("People runtime not supported"),
        }
    }

    pub fn default_system_chains(&self) -> Vec<Self> {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => vec![Self::AssetHubPolkadot, Self::PeoplePolkadot],
            #[cfg(feature = "kusama")]
            Self::Kusama => vec![Self::AssetHubKusama, Self::PeopleKusama],
            #[cfg(feature = "paseo")]
            Self::Paseo => vec![Self::AssetHubPaseo, Self::PeoplePaseo],
            #[cfg(feature = "westend")]
            Self::Westend => vec![Self::AssetHubWestend, Self::PeopleWestend],
            _ => panic!("Unsupported relay-chain"),
        }
    }

    pub fn chain_specs(&self) -> &str {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => POLKADOT_SPEC,
            #[cfg(feature = "kusama")]
            Self::Kusama => KUSAMA_SPEC,
            #[cfg(feature = "westend")]
            Self::Westend => WESTEND_SPEC,
            #[cfg(feature = "paseo")]
            Self::Paseo => PASEO_SPEC,
            #[cfg(feature = "polkadot")]
            Self::PeoplePolkadot => PEOPLE_POLKADOT_SPEC,
            #[cfg(feature = "kusama")]
            Self::PeopleKusama => PEOPLE_KUSAMA_SPEC,
            #[cfg(feature = "westend")]
            Self::PeopleWestend => PEOPLE_WESTEND_SPEC,
            #[cfg(feature = "paseo")]
            Self::PeoplePaseo => PEOPLE_PASEO_SPEC,
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => ASSET_HUB_POLKADOT_SPEC,
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => ASSET_HUB_KUSAMA_SPEC,
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => ASSET_HUB_WESTEND_SPEC,
            #[cfg(feature = "paseo")]
            Self::AssetHubPaseo => ASSET_HUB_PASEO_SPEC,
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn chain_genesis_hash(&self) -> H256 {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => {
                H256::from_str("0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "kusama")]
            Self::Kusama => {
                H256::from_str("0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "westend")]
            Self::Westend => {
                H256::from_str("0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "paseo")]
            Self::Paseo => {
                H256::from_str("0x374057be67b355151f271ff70c3db98308c62c8adc48dc6724b6a009a1a014fd")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "polkadot")]
            Self::PeoplePolkadot => {
                H256::from_str("0x67fa177a097bfa18f77ea95ab56e9bcdfeb0e5b8a40e46298bb93e16b6fc5008")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "kusama")]
            Self::PeopleKusama => {
                H256::from_str("0xc1af4cb4eb3918e5db15086c0cc5ec17fb334f728b7c65dd44bfe1e174ff8b3f")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "westend")]
            Self::PeopleWestend => {
                H256::from_str("0x1eb6fb0ba5187434de017a70cb84d4f47142df1d571d0ef9e7e1407f2b80b93c")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "paseo")]
            Self::PeoplePaseo => {
                H256::from_str("0xe6c30d6e148f250b887105237bcaa5cb9f16dd203bf7b5b9d4f1da7387cb86ec")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => {
                H256::from_str("0x68d56f15f85d3136970ec16946040bc1752654e906147f7e43e9d539d7c3de2f")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => {
                H256::from_str("0x48239ef607d7928874027a43a67689209727dfb3d3dc5e5b03a39bdc2eda771a")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => {
                H256::from_str("0x67f9723393ef76214df0118c34bbbd3dbebc8ed46a10973a8c969d48fe7598c9")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "paseo")]
            Self::AssetHubPaseo => {
                H256::from_str("0xd6eec26135305a8ad257a20d003357284c8aa03d0bdb2b357ab0a22371e11ef2")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "polkadot")]
            Self::BridgeHubPolkadot => {
                H256::from_str("0xdcf691b5a3fbe24adc99ddc959c0561b973e329b1aef4c4b22e7bb2ddecb4464")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "kusama")]
            Self::BridgeHubKusama => {
                H256::from_str("0x00dcb981df86429de8bbacf9803401f09485366c44efbf53af9ecfab03adc7e5")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "westend")]
            Self::BridgeHubWestend => {
                H256::from_str("0x0441383e31d1266a92b4cb2ddd4c2e3661ac476996db7e5844c52433b81fe782")
                    .expect("Invalid genesis hash")
            }
            #[cfg(feature = "paseo")]
            Self::BridgeHubPaseo => {
                H256::from_str("0xcc624979479dc37afee4cb23cb72b1772bbf377c0d3e8fa257c0fe6146572e3e")
                    .expect("Invalid genesis hash")
            }
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn chain_state_root_hash(&self) -> H256 {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => get_state_root_hash(POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::Kusama => get_state_root_hash(KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::Westend => get_state_root_hash(WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::Paseo => get_state_root_hash(PASEO_SPEC),
            #[cfg(feature = "polkadot")]
            Self::PeoplePolkadot => get_state_root_hash(PEOPLE_POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::PeopleKusama => get_state_root_hash(PEOPLE_KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::PeopleWestend => get_state_root_hash(PEOPLE_WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::PeoplePaseo => get_state_root_hash(PEOPLE_PASEO_SPEC),
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => get_state_root_hash(ASSET_HUB_POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => get_state_root_hash(ASSET_HUB_KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => get_state_root_hash(ASSET_HUB_WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::AssetHubPaseo => get_state_root_hash(ASSET_HUB_PASEO_SPEC),
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn account_format(&self) -> u16 {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot | Self::AssetHubPolkadot | Self::PeoplePolkadot => 0,
            #[cfg(feature = "kusama")]
            Self::Kusama | Self::AssetHubKusama | Self::PeopleKusama => 2,
            #[cfg(feature = "westend")]
            Self::Westend | Self::AssetHubWestend | Self::PeopleWestend => 42,
            #[cfg(feature = "paseo")]
            Self::Paseo | Self::AssetHubPaseo | Self::PeoplePaseo => 42,
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn _account_format(&self) -> u16 {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => get_ss58_format(POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::Kusama => get_ss58_format(KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::Westend => get_ss58_format(WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::Paseo => get_ss58_format(PASEO_SPEC),
            #[cfg(feature = "polkadot")]
            Self::PeoplePolkadot => get_ss58_format(PEOPLE_POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::PeopleKusama => get_ss58_format(PEOPLE_KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::PeopleWestend => get_ss58_format(PEOPLE_WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::PeoplePaseo => get_ss58_format(PEOPLE_PASEO_SPEC),
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => get_ss58_format(ASSET_HUB_POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => get_ss58_format(ASSET_HUB_KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => get_ss58_format(ASSET_HUB_WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::AssetHubPaseo => get_ss58_format(ASSET_HUB_PASEO_SPEC),
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn token_symbol(&self) -> &'static str {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot | Self::AssetHubPolkadot | Self::PeoplePolkadot => "DOT",
            #[cfg(feature = "kusama")]
            Self::Kusama | Self::AssetHubKusama | Self::PeopleKusama => "KSM",
            #[cfg(feature = "westend")]
            Self::Westend | Self::AssetHubWestend | Self::PeopleWestend => "WND",
            #[cfg(feature = "paseo")]
            Self::Paseo | Self::AssetHubPaseo | Self::PeoplePaseo => "PAS",
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn _token_symbol(&self) -> String {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => get_symbol(POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::Kusama => get_symbol(KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::Westend => get_symbol(WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::Paseo => get_symbol(PASEO_SPEC),
            #[cfg(feature = "polkadot")]
            Self::PeoplePolkadot => get_symbol(PEOPLE_POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::PeopleKusama => get_symbol(PEOPLE_KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::PeopleWestend => get_symbol(PEOPLE_WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::PeoplePaseo => get_symbol(PEOPLE_PASEO_SPEC),
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => get_symbol(ASSET_HUB_POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => get_symbol(ASSET_HUB_KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => get_symbol(ASSET_HUB_WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::AssetHubPaseo => get_symbol(ASSET_HUB_PASEO_SPEC),
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn token_decimals(&self) -> u32 {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot | Self::AssetHubPolkadot | Self::PeoplePolkadot => 10,
            #[cfg(feature = "kusama")]
            Self::Kusama | Self::AssetHubKusama | Self::PeopleKusama => 12,
            #[cfg(feature = "westend")]
            Self::Westend | Self::AssetHubWestend | Self::PeopleWestend => 12,
            #[cfg(feature = "paseo")]
            Self::Paseo | Self::AssetHubPaseo | Self::PeoplePaseo => 10,
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn _token_decimals(&self) -> u32 {
        match &self {
            #[cfg(feature = "polkadot")]
            Self::Polkadot => get_decimals(POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::Kusama => get_decimals(KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::Westend => get_decimals(WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::Paseo => get_decimals(PASEO_SPEC),
            #[cfg(feature = "polkadot")]
            Self::PeoplePolkadot => get_decimals(PEOPLE_POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::PeopleKusama => get_decimals(PEOPLE_KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::PeopleWestend => get_decimals(PEOPLE_WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::PeoplePaseo => get_decimals(PEOPLE_PASEO_SPEC),
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => get_decimals(ASSET_HUB_POLKADOT_SPEC),
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => get_decimals(ASSET_HUB_KUSAMA_SPEC),
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => get_decimals(ASSET_HUB_WESTEND_SPEC),
            #[cfg(feature = "paseo")]
            Self::AssetHubPaseo => get_decimals(ASSET_HUB_PASEO_SPEC),
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn is_relay_chain(&self) -> bool {
        match self {
            Self::Local => true,
            #[cfg(feature = "polkadot")]
            Self::Polkadot => true,
            #[cfg(feature = "kusama")]
            Self::Kusama => true,
            #[cfg(feature = "paseo")]
            Self::Paseo => true,
            #[cfg(feature = "westend")]
            Self::Westend => true,
            _ => false,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "Local",
            #[cfg(feature = "polkadot")]
            Self::Polkadot => "Polkadot",
            #[cfg(feature = "kusama")]
            Self::Kusama => "Kusama",
            #[cfg(feature = "westend")]
            Self::Westend => "Westend",
            #[cfg(feature = "paseo")]
            Self::Paseo => "Paseo",
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => "Polkadot Hub",
            #[cfg(feature = "polkadot")]
            Self::BridgeHubPolkadot => "Polkadot Bridge",
            #[cfg(feature = "polkadot")]
            Self::PeoplePolkadot => "Polkadot People",
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => "Kusama Hub",
            #[cfg(feature = "kusama")]
            Self::BridgeHubKusama => "Kusama Bridge",
            #[cfg(feature = "kusama")]
            Self::PeopleKusama => "Kusama People",
            #[cfg(feature = "paseo")]
            Self::AssetHubPaseo => "Paseo Hub",
            #[cfg(feature = "paseo")]
            Self::BridgeHubPaseo => "Paseo Bridge",
            #[cfg(feature = "paseo")]
            Self::PeoplePaseo => "Paseo People",
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => "Westend Hub",
            #[cfg(feature = "westend")]
            Self::BridgeHubWestend => "Westend Bridge",
            #[cfg(feature = "westend")]
            Self::PeopleWestend => "Westend People",
        }
    }

    pub fn as_str_long(&self) -> &'static str {
        match self {
            Self::Local => "Local",
            #[cfg(feature = "polkadot")]
            Self::Polkadot => "Polkadot",
            #[cfg(feature = "kusama")]
            Self::Kusama => "Kusama",
            #[cfg(feature = "westend")]
            Self::Westend => "Westend",
            #[cfg(feature = "paseo")]
            Self::Paseo => "Paseo",
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => "Asset Hub Polkadot",
            #[cfg(feature = "polkadot")]
            Self::BridgeHubPolkadot => "Bridge Hub Polkadot",
            #[cfg(feature = "polkadot")]
            Self::PeoplePolkadot => "People Polkadot",
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => "Asset Hub Kusama",
            #[cfg(feature = "kusama")]
            Self::BridgeHubKusama => "Bridge Hub Kusama",
            #[cfg(feature = "kusama")]
            Self::PeopleKusama => "People Kusama",
            #[cfg(feature = "paseo")]
            Self::AssetHubPaseo => "Asset Hub Paseo",
            #[cfg(feature = "paseo")]
            Self::BridgeHubPaseo => "Bridge Hub Paseo",
            #[cfg(feature = "paseo")]
            Self::PeoplePaseo => "People Paseo",
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => "Asset Hub Westend",
            #[cfg(feature = "westend")]
            Self::BridgeHubWestend => "Bridge Hub Westend",
            #[cfg(feature = "westend")]
            Self::PeopleWestend => "People Westend",
        }
    }

    pub fn chain_name(&self) -> String {
        self.as_str_long().to_lowercase().replace(' ', "-")
    }

    pub fn legacy_name(&self) -> String {
        match self {
            #[cfg(feature = "polkadot")]
            Self::AssetHubPolkadot => "statemint".to_string(),
            #[cfg(feature = "kusama")]
            Self::AssetHubKusama => "statemine".to_string(),
            #[cfg(feature = "westend")]
            Self::AssetHubWestend => "westmint".to_string(),
            _ => self.chain_name(),
        }
    }

    pub fn log_block_hash_explorer(&self, block_hash: H256) {
        let config = CONFIG.clone();
        let hash = format!("{:#x}", block_hash);
        if let Some(url) = config.explorer_url(&self.chain_name(), &hash) {
            info!("{url}");
        };
    }

    pub fn signer_account_id(&self) -> Result<AccountId32, Error> {
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            for (chain_name, chain_config) in chain {
                if self == chain_name {
                    let Ok(account_id) = chain_config.signer_account_id() else {
                        return config.signer_account_id();
                    };
                    return Ok(account_id);
                }
            }
        }
        Err(Error::SignerNotDefined)
    }

    pub fn is_qrcode_enabled(&self) -> bool {
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            for (chain_name, chain_config) in chain {
                if self == chain_name {
                    return if chain_config.signer.is_some() {
                        chain_config.is_qrcode_enabled()
                    } else {
                        config.is_qrcode_enabled()
                    };
                }
            }
        }
        config.is_qrcode_enabled()
    }
}

impl std::fmt::Display for SupportedRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for SupportedRuntime {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "polkadot")]
            "polkadot" => Ok(SupportedRuntime::Polkadot),
            #[cfg(feature = "kusama")]
            "kusama" => Ok(SupportedRuntime::Kusama),
            #[cfg(feature = "paseo")]
            "paseo" => Ok(SupportedRuntime::Paseo),
            #[cfg(feature = "westend")]
            "westend" => Ok(SupportedRuntime::Westend),
            #[cfg(feature = "polkadot")]
            "asset_hub_polkadot" => Ok(SupportedRuntime::AssetHubPolkadot),
            #[cfg(feature = "polkadot")]
            "bridge_hub_polkadot" => Ok(SupportedRuntime::BridgeHubPolkadot),
            #[cfg(feature = "polkadot")]
            "people_polkadot" => Ok(SupportedRuntime::PeoplePolkadot),
            #[cfg(feature = "kusama")]
            "asset_hub_kusama" => Ok(SupportedRuntime::AssetHubKusama),
            #[cfg(feature = "kusama")]
            "bridge_hub_kusama" => Ok(SupportedRuntime::BridgeHubKusama),
            #[cfg(feature = "kusama")]
            "people_kusama" => Ok(SupportedRuntime::PeopleKusama),
            #[cfg(feature = "paseo")]
            "asset_hub_paseo" => Ok(SupportedRuntime::AssetHubPaseo),
            #[cfg(feature = "paseo")]
            "bridge_hub_paseo" => Ok(SupportedRuntime::BridgeHubPaseo),
            #[cfg(feature = "paseo")]
            "people_paseo" => Ok(SupportedRuntime::PeoplePaseo),
            #[cfg(feature = "westend")]
            "asset_hub_westend" => Ok(SupportedRuntime::AssetHubWestend),
            #[cfg(feature = "westend")]
            "bridge_hub_westend" => Ok(SupportedRuntime::BridgeHubWestend),
            #[cfg(feature = "westend")]
            "people_westend" => Ok(SupportedRuntime::PeopleWestend),
            _ => Err(Error::UnsupportedChain(s.to_string())),
        }
    }
}

fn get_state_root_hash(chain_specs: &str) -> H256 {
    let spec: serde_json::Result<Value> = serde_json::from_str(chain_specs);
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
    let spec: serde_json::Result<Value> = serde_json::from_str(chain_specs);
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
    let spec: serde_json::Result<Value> = serde_json::from_str(chain_specs);
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
    let spec: serde_json::Result<Value> = serde_json::from_str(chain_specs);
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
