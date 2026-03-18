use crate::error::Error;
use crate::runtime::SupportedRuntime;
use crate::themes::{default_active_theme, Themes};
use lazy_static::lazy_static;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use subxt::utils::AccountId32;
use suno_theme::Theme;

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

type Stash = AccountId32;

/// Provides default value for the configuration file path
fn default_config_path() -> &'static str {
    ".config.yaml"
}

/// Provides default value for the proxy account file path
fn default_proxy_path() -> String {
    ".proxy_account.json".to_string()
}

/// Provides default value for Themes struct
fn default_themes() -> Themes {
    Themes::default()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub chains: Vec<HashMap<SupportedRuntime, ChainConfig>>,
    // TODO: Add support for RPCs
    // rpcs: Vec<HashMap<String, Vec<String>>>,
    pub features: Features,
    pub signer: Option<Signer>,
    pub explorer: Explorer,
    #[serde(default = "default_themes")]
    themes: Themes,
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
    #[serde(default = "default_proxy_path")]
    proxy_path: String,
}

impl Signer {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();

        if !path.exists() {
            warn!("Proxy path does not exist: {}", path.display());
            return Err(Error::InvalidPath(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;
        if content.is_empty() {
            warn!("Proxy path content is empty: {}", path.display());
            return Err(Error::InvalidContent(path.display().to_string()));
        }

        Ok(Signer {
            proxy_path: path.to_string_lossy().into_owned(),
        })
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Explorer {
    url: Option<String>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Error::InvalidPath(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;
        if content.is_empty() {
            return Err(Error::InvalidContent(path.display().to_string()));
        }

        let mut config: Config = serde_yaml::from_str(&content)?;

        // Verify and validate if signer path exists
        if let Some(signer) = config.signer {
            let signer = Signer::from_file(signer.proxy_path).ok();
            config.signer = signer;
        }

        // Load themes
        let themes = Themes::load(&config.themes.path)?;
        config.themes.set_themes(themes);

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), Error> {
        // Validate that at least one chain is configured
        if self.chains.is_empty() {
            return Err(Error::ChainNotAvailable);
        }

        // Validate themes
        self.themes.validate()?;

        Ok(())
    }

    pub fn signer_path(&self) -> Option<String> {
        self.signer.as_ref().map(|s| s.proxy_path.clone())
    }

    pub fn set_signer_path(&mut self, path: &str) {
        let signer = Signer::from_file(path).ok();
        self.signer = signer;
    }

    pub fn explorer_url(&self, chain: &str, block_hash: &str) -> Option<String> {
        self.explorer.url.as_ref().map(|url| {
            url.replace("{chain}", chain)
                .replace("{block_hash}", block_hash)
        })
    }

    pub fn theme(&self) -> &Theme {
        self.themes.theme()
    }

    fn set_default_theme(&mut self) {
        self.themes.active = default_active_theme();
    }
}

fn get_config() -> Result<Config, Error> {
    let default_config_path = default_config_path();

    let matches = clap::Command::new("suno")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::new("config-path")
                .short('c')
                .long("config-path")
                .value_name("FILE")
                .default_value(default_config_path)
                .help("Sets a custom config file path."),
        )
        .arg(
            clap::Arg::new("proxy-path")
                .short('p')
                .long("proxy-path")
                .value_name("FILE")
                .help("Sets a custom proxy account file path."),
        )
        .get_matches();

    let config_path = matches
        .get_one::<String>("config-path")
        .map(|s| s.as_str())
        .unwrap_or(default_config_path);

    info!("Loading configuration from {}", config_path);

    // Read and parse the config file
    let mut config = Config::from_file(config_path)?;

    // If not specified in the config file, load the signer proxy path
    // from the command line argument, otherwise try to load the default
    if config.signer.is_none() {
        if let Some(path) = matches.get_one::<String>("proxy-path") {
            config.set_signer_path(path.as_str());
        }
    }

    if config.themes.active.is_empty() {
        config.set_default_theme();
    }

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
        assert_eq!(SupportedRuntime::Local.as_str_long(), "Local");
        assert_eq!(SupportedRuntime::Polkadot.as_str_long(), "Polkadot");
        assert_eq!(SupportedRuntime::Kusama.as_str_long(), "Kusama");
        assert_eq!(SupportedRuntime::Paseo.as_str_long(), "Paseo");
        assert_eq!(
            SupportedRuntime::AssetHubPolkadot.as_str_long(),
            "Asset Hub Polkadot"
        );
    }

    #[test]
    fn test_config_validation_empty_chains() {
        let config = Config {
            chains: vec![],
            features: Features::default(),
            signer: None,
            explorer: Explorer::default(),
            themes: Themes::default(),
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
themes:
  active: "Suno Light"
  path: "./themes"
signer:
  proxy_path: ".proxy_private.seed"
explorer:
  url: "https://polkadot.js.org/apps/?rpc=wss://{chain}.rpc.turboflakes.io#/explorer/query/{block_hash}"
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
themes:
  active: "Suno Dark"
  path: "./themes"
signer:
  proxy_path: ".proxy_private.json"
explorer:
    url: "https://polkadot.js.org/apps/?rpc=wss://{chain}.rpc.turboflakes.io#/explorer/query/{block_hash}"
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
