use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

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
    Polkadot,
    Kusama,
    Paseo,
    #[serde(rename = "asset-hub-polkadot")]
    AssetHubPolkadot,
}

impl std::fmt::Display for SupportedRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polkadot => write!(f, "Polkadot"),
            Self::Kusama => write!(f, "Kusama"),
            Self::Paseo => write!(f, "Paseo"),
            Self::AssetHubPolkadot => write!(f, "Asset Hub Polkadot"),
        }
    }
}

// impl SupportedRuntime {
//     pub fn default_rpc_url(&self) -> &'static str {
//         let config = CONFIG.clone();
//         config
//             .rpcs
//             .iter()
//             .find_map(|rpc_chain| {
//                 rpc_chain
//                     .chain
//                     .get(self)
//                     .map(|urls| urls.first().unwrap().as_str())
//             })
//             .unwrap()
//         // .unwrap_or_else(|| match &self {
//         //     Self::Polkadot => "wss://rpc.ibp.network:443/polkadot",
//         //     Self::Kusama => "wss://rpc.ibp.network:443/kusama",
//         //     Self::Paseo => "wss://rpc.ibp.network:443/paseo",
//         //     Self::AssetHubPolkadot => "wss://rpc.ibp.network:443/asset-hub-polkadot",
//         // })
//     }
// }

type Stash = String;
type Stashes = Vec<Stash>;

#[derive(Debug, Serialize, Deserialize)]
pub struct StashChain {
    #[serde(flatten)]
    chain: HashMap<SupportedRuntime, Stashes>,
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
pub struct RpcChain {
    #[serde(flatten)]
    chain: HashMap<SupportedRuntime, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub chains: Vec<SupportedRuntime>,
    pub validators: Vec<StashChain>,
    pub collators: Vec<StashChain>,
    pub rpcs: Vec<RpcChain>,
    pub features: Features,
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

    pub fn get_default_rpc_url(&self, runtime: &SupportedRuntime) -> Option<&str> {
        self.rpcs.iter().find_map(|rpc_chain| {
            rpc_chain
                .chain
                .get(runtime)
                .map(|urls| urls.first().unwrap().as_str())
        })
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
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_chain_serialization() {
        let chain = SupportedRuntime::Polkadot;
        let serialized = serde_yaml::to_string(&chain).unwrap();
        assert_eq!(serialized.trim(), "polkadot");

        let chain = SupportedRuntime::AssetHubPolkadot;
        let serialized = serde_yaml::to_string(&chain).unwrap();
        assert_eq!(serialized.trim(), "asset-hub-polkadot");
    }

    #[test]
    fn test_chain_deserialization() {
        let chain: SupportedRuntime = serde_yaml::from_str("polkadot").unwrap();
        assert_eq!(chain, SupportedRuntime::Polkadot);

        let chain: SupportedRuntime = serde_yaml::from_str("asset-hub-polkadot").unwrap();
        assert_eq!(chain, SupportedRuntime::AssetHubPolkadot);
    }

    #[test]
    fn test_valid_config_from_yaml() {
        let yaml = r#"
chains:
  - polkadot
  - kusama
validators:
  - polkadot: ["stash_1", "stash_2"]
  - kusama: ["stash_3"]
collators:
  - polkadot: ["stash_4"]
  - kusama: ["stash_5", "stash_6"]
rpcs:
    - polkadot: ["endpoint_1"]
    - kusama: ["endpoint_2", "endpoint_3"]
features:
  enable_validators: true
  enable_collators: true
  enable_rpcs: false
"#;
        let file = create_temp_file(yaml);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.chains.len(), 2);
        assert_eq!(config.validators.len(), 2);
        assert_eq!(config.validators.len(), 2);
        assert_eq!(config.collators.len(), 2);
        assert_eq!(config.rpcs.len(), 2);
        assert_eq!(
            config.rpcs[0]
                .chain
                .get(&SupportedRuntime::Polkadot)
                .unwrap(),
            &vec!["endpoint_1".to_string()]
        );
        assert_eq!(
            config.rpcs[1].chain.get(&SupportedRuntime::Kusama).unwrap(),
            &vec!["endpoint_2".to_string(), "endpoint_3".to_string()]
        );
        assert!(config.features.enable_validators);
        assert!(config.features.enable_collators);
        assert!(!config.features.enable_rpcs);
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let config = Config {
            chains: vec![SupportedRuntime::Polkadot],
            validators: vec![],
            collators: vec![],
            rpcs: vec![],
            features: Features {
                enable_validators: true,
                enable_collators: true,
                enable_rpcs: true,
            },
        };
        assert!(config.validate().is_ok());

        // Invalid config (no chains)
        let config = Config {
            chains: vec![],
            validators: vec![],
            collators: vec![],
            rpcs: vec![],
            features: Features {
                enable_validators: true,
                enable_collators: true,
                enable_rpcs: true,
            },
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_yaml_format() {
        let invalid_yaml = "invalid: - yaml: content";
        let file = create_temp_file(invalid_yaml);
        assert!(Config::from_file(file.path()).is_err());
    }

    #[test]
    fn test_stash_chain_structure() {
        let mut map = HashMap::new();
        map.insert(SupportedRuntime::Polkadot, vec!["stash_1".to_string()]);

        let stash_chain = StashChain { chain: map };

        let serialized = serde_yaml::to_string(&stash_chain).unwrap();
        let deserialized: StashChain = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(
            deserialized.chain.get(&SupportedRuntime::Polkadot).unwrap(),
            &vec!["stash_1".to_string()]
        );
    }

    // Helper function to create temporary file with content
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }
}
