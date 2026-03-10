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

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Ord, PartialOrd, Debug)]
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

    pub fn chain_genesis_hash(&self) -> H256 {
        match &self {
            Self::Polkadot => {
                H256::from_str("0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3")
                    .expect("Invalid genesis hash")
            }
            Self::Kusama => {
                H256::from_str("0xb0a8d493285c2df73290dfb7e61f870f17b41801197a149ca93654499ea3dafe")
                    .expect("Invalid genesis hash")
            }
            Self::Westend => {
                H256::from_str("0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e")
                    .expect("Invalid genesis hash")
            }
            Self::Paseo => {
                H256::from_str("0x77afd6190f1554ad45fd0d31aee62aacc33c6db0ea801129acb813f913e0764f")
                    .expect("Invalid genesis hash")
            }
            Self::PeoplePolkadot => {
                H256::from_str("0x67fa177a097bfa18f77ea95ab56e9bcdfeb0e5b8a40e46298bb93e16b6fc5008")
                    .expect("Invalid genesis hash")
            }
            Self::PeopleKusama => {
                H256::from_str("0xc1af4cb4eb3918e5db15086c0cc5ec17fb334f728b7c65dd44bfe1e174ff8b3f")
                    .expect("Invalid genesis hash")
            }
            Self::PeopleWestend => {
                H256::from_str("0x1eb6fb0ba5187434de017a70cb84d4f47142df1d571d0ef9e7e1407f2b80b93c")
                    .expect("Invalid genesis hash")
            }
            Self::PeoplePaseo => {
                H256::from_str("0xe6c30d6e148f250b887105237bcaa5cb9f16dd203bf7b5b9d4f1da7387cb86ec")
                    .expect("Invalid genesis hash")
            }
            Self::AssetHubPolkadot => {
                H256::from_str("0x68d56f15f85d3136970ec16946040bc1752654e906147f7e43e9d539d7c3de2f")
                    .expect("Invalid genesis hash")
            }
            Self::AssetHubKusama => {
                H256::from_str("0x48239ef607d7928874027a43a67689209727dfb3d3dc5e5b03a39bdc2eda771a")
                    .expect("Invalid genesis hash")
            }
            Self::AssetHubWestend => {
                H256::from_str("0x67f9723393ef76214df0118c34bbbd3dbebc8ed46a10973a8c969d48fe7598c9")
                    .expect("Invalid genesis hash")
            }
            Self::AssetHubPaseo => {
                H256::from_str("0xd6eec26135305a8ad257a20d003357284c8aa03d0bdb2b357ab0a22371e11ef2")
                    .expect("Invalid genesis hash")
            }
            Self::BridgeHubPolkadot => {
                H256::from_str("0xdcf691b5a3fbe24adc99ddc959c0561b973e329b1aef4c4b22e7bb2ddecb4464")
                    .expect("Invalid genesis hash")
            }
            Self::BridgeHubKusama => {
                H256::from_str("0x00dcb981df86429de8bbacf9803401f09485366c44efbf53af9ecfab03adc7e5")
                    .expect("Invalid genesis hash")
            }
            Self::BridgeHubWestend => {
                H256::from_str("0x0441383e31d1266a92b4cb2ddd4c2e3661ac476996db7e5844c52433b81fe782")
                    .expect("Invalid genesis hash")
            }
            Self::BridgeHubPaseo => {
                H256::from_str("0xcc624979479dc37afee4cb23cb72b1772bbf377c0d3e8fa257c0fe6146572e3e")
                    .expect("Invalid genesis hash")
            }
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
            Self::Polkadot | Self::AssetHubPolkadot | Self::PeoplePolkadot => 0,
            Self::Kusama | Self::AssetHubKusama | Self::PeopleKusama => 2,
            Self::Westend | Self::AssetHubWestend | Self::PeopleWestend => 42,
            Self::Paseo | Self::AssetHubPaseo | Self::PeoplePaseo => 0,
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn _account_format(&self) -> u16 {
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

    pub fn token_symbol(&self) -> &'static str {
        match &self {
            Self::Polkadot | Self::AssetHubPolkadot | Self::PeoplePolkadot => "DOT",
            Self::Kusama | Self::AssetHubKusama | Self::PeopleKusama => "KSM",
            Self::Westend | Self::AssetHubWestend | Self::PeopleWestend => "WND",
            Self::Paseo | Self::AssetHubPaseo | Self::PeoplePaseo => "PAS",
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn _token_symbol(&self) -> String {
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
            Self::Polkadot | Self::AssetHubPolkadot | Self::PeoplePolkadot => 10,
            Self::Kusama | Self::AssetHubKusama | Self::PeopleKusama => 12,
            Self::Westend | Self::AssetHubWestend | Self::PeopleWestend => 12,
            Self::Paseo | Self::AssetHubPaseo | Self::PeoplePaseo => 10,
            _ => panic!("Unsupported chain"),
        }
    }

    pub fn _token_decimals(&self) -> u32 {
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
        matches!(
            self,
            Self::Local | Self::Polkadot | Self::Kusama | Self::Paseo | Self::Westend
        )
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
