use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use subxt::utils::AccountId32;

pub type ChainPrefix = u16;
pub type ChainTokenSymbol = String;

// Set Config struct into a CONFIG lazy_static to avoid multiple processing
lazy_static! {
    pub static ref CONFIG: Arc<Config> = {
        match get_config() {
            Ok(config) => Arc::new(config),
            Err(e) => {
                eprintln!("Failed to load configuration: {}", e);
                std::process::exit(1);
            }
        }
    };
}

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

pub type Stash = AccountId32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub chains: Vec<HashMap<SupportedRuntime, ChainConfig>>,
    // TODO: Add support for RPCs
    // rpcs: Vec<HashMap<String, Vec<String>>>,
    pub features: Features,
    pub signer: Signer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub light_client: bool,
    #[serde(default)]
    pub validators: Vec<NodeConfig>,
    #[serde(default)]
    pub collators: Vec<NodeConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeConfig {
    Address(Stash),
    Detailed {
        stash: Stash,
        commands: Option<Vec<Command>>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub run: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    pub enable_validators: bool,
    pub enable_collators: bool,
    pub enable_rpcs: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            enable_validators: true,
            enable_collators: false,
            enable_rpcs: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signer {
    proxy_seed_path: String,
}

impl Default for Signer {
    fn default() -> Self {
        Self {
            proxy_seed_path: ".proxy_private.seed".into(),
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate a chain is enabled
        if self.chains.is_empty() {
            return Err(
                "At least one chain has to be enabled [Polkadot, Kusama, Paseo]".to_string(),
            );
        }

        Ok(())
    }

    pub fn signer_path(&self) -> String {
        self.signer.proxy_seed_path.clone()
    }
}

fn get_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Check for custom config file path in environment variable
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| ".config.yaml".to_string());

    // Read and parse the config file
    let config = Config::from_file(&config_path)?;

    // Validate the configuration
    config.validate()?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_address_only() {
        let yaml = r#""5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY""#;
        let config: NodeConfig = serde_yaml::from_str(yaml).unwrap();

        match config {
            NodeConfig::Address(account_id) => {
                assert_eq!(
                    account_id.to_string(),
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                );
            }
            _ => panic!("Expected Address variant"),
        }
    }

    #[test]
    fn test_detailed_with_commands() {
        let yaml = r#"
            stash: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            commands:
              - name: Ping
                run: "echo 'Ping'"
        "#;
        let config: NodeConfig = serde_yaml::from_str(yaml).unwrap();

        match config {
            NodeConfig::Detailed { stash, commands } => {
                assert_eq!(
                    stash.to_string(),
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                );
                let commands = commands.unwrap();
                assert_eq!(commands[0].name, "Ping");
                assert_eq!(commands[0].run, "echo 'Ping'");
            }
            _ => panic!("Expected Detailed variant"),
        }
    }

    #[test]
    fn test_detailed_without_commands() {
        let yaml = r#"
                stash: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            "#;
        let config: NodeConfig = serde_yaml::from_str(yaml).unwrap();

        match config {
            NodeConfig::Detailed { stash, commands } => {
                assert_eq!(
                    stash.to_string(),
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                );
                assert!(commands.is_none());
            }
            _ => panic!("Expected Detailed variant"),
        }
    }

    #[test]
    fn test_invalid_account_id() {
        let yaml = r#""invalid_account_id""#;
        assert!(serde_yaml::from_str::<NodeConfig>(yaml).is_err());
    }

    #[test]
    fn test_supported_runtime_display() {
        assert_eq!(SupportedRuntime::Local.to_string(), "Local");
        assert_eq!(SupportedRuntime::Polkadot.to_string(), "Polkadot");
        assert_eq!(SupportedRuntime::Kusama.to_string(), "Kusama");
        assert_eq!(SupportedRuntime::Paseo.to_string(), "Paseo");
        assert_eq!(
            SupportedRuntime::AssetHubPolkadot.to_string(),
            "AssetHub Polkadot"
        );
    }

    #[test]
    fn test_config_validation_empty_chains() {
        let config = Config {
            chains: vec![],
            features: Features::default(),
            signer: Signer::default(),
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_from_file_valid() {
        let yaml = r#"
chains:
  - polkadot:
      rpc_url: "wss://rpc.polkadot.io"
      light_client: false
      validators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
features:
  enable_validators: true
  enable_collators: false
  enable_rpcs: false
signer:
  proxy_seed_path: ".proxy_private.seed"
"#;
        let file = create_temp_file(yaml);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.chains.len(), 1);
        assert!(config.features.enable_validators);
    }

    #[test]
    fn test_supported_runtime_serialization() {
        let runtime = SupportedRuntime::AssetHubPolkadot;
        let serialized = serde_yaml::to_string(&runtime).unwrap();
        assert_eq!(serialized.trim(), "asset_hub_polkadot");

        let deserialized: SupportedRuntime = serde_yaml::from_str("asset_hub_polkadot").unwrap();
        assert!(matches!(deserialized, SupportedRuntime::AssetHubPolkadot));
    }

    #[test]
    fn test_config_with_all_chain_types() {
        let yaml = r#"
chains:
  - polkadot:
      rpc_url: "wss://rpc.polkadot.io"
      light_client: false
      validators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
  - kusama:
      rpc_url: "wss://rpc.kusama.io"
      light_client: false
      validators:
        - stash: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
          commands:
            - name: "test"
              run: "echo test"
  - asset_hub_polkadot:
      rpc_url: "wss://rpc.asset.hub"
      light_client: false
      collators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
features:
  enable_validators: true
  enable_collators: true
  enable_rpcs: false
signer:
  proxy_seed_path: ".proxy_private.seed"
"#;

        let file = create_temp_file(yaml);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.chains.len(), 3);
        assert!(config.features.enable_validators);
        assert!(config.features.enable_collators);
    }

    // Helper function to create temporary file with content
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }
}
