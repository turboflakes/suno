use crate::access::SshConfig;
use crate::custom::CustomCommand;
use crate::error::Error;
use crate::logs::Logs;
use crate::runtime::SupportedRuntime;
use crate::signer::{default_proxy_path, Signer};
use crate::themes::{default_active_theme, Themes};
use crate::vault::Vault;
use lazy_static::lazy_static;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::str::FromStr;
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

type Chains = Vec<HashMap<SupportedRuntime, ChainConfig>>;
type Stash = AccountId32;
type Pat = String;

/// Provides default value for Themes struct
fn default_themes() -> Themes {
    Themes::default()
}

/// Provides default value for Features struct
fn default_features() -> Features {
    Features::default()
}

/// Provides default value for Explorer struct
fn default_explorer() -> Explorer {
    Explorer::default()
}

/// Provides default value for Logs struct
fn default_logs() -> Logs {
    Logs::default()
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Subcommand {
    Update { version: Option<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// CLI subcommand to execute.
    #[serde(default)]
    pub subcommand: Option<Subcommand>,
    /// Chain configurations.
    #[serde(default)]
    pub chains: Chains,
    /// Features to enable or disable.
    #[serde(default = "default_features")]
    pub features: Features,
    /// Signer configuration.
    #[serde(default)]
    pub signer: Option<Signer>,
    /// Vault configuration.
    #[serde(default)]
    pub vault: Option<Vault>,
    /// Explorer configuration.
    #[serde(default = "default_explorer")]
    pub explorer: Explorer,
    /// Themes configuration.
    #[serde(default = "default_themes")]
    pub themes: Themes,
    /// Logs configuration.
    #[serde(default = "default_logs")]
    pub logs: Logs,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            subcommand: None,
            chains: Vec::new(),
            features: default_features(),
            signer: None,
            vault: None,
            explorer: default_explorer(),
            themes: default_themes(),
            logs: default_logs(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    /// URL to fetch data from.
    #[serde(default)]
    url: String,
    /// Personal Access Token for the URL.
    #[serde(default)]
    pat: Option<Pat>,
}

impl Source {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            pat: None,
        }
    }

    pub fn with(url: String, pat: Option<Pat>) -> Self {
        Self { url, pat }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn pat(&self) -> &Option<Pat> {
        &self.pat
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainConfig {
    /// RPC URL for the chain.
    #[serde(default)]
    pub rpc_url: String,
    /// Signer configuration for the chain.
    #[serde(default)]
    pub signer: Option<Signer>,
    /// Validators stashes for the chain.
    #[serde(default)]
    pub validators: Vec<NodeConfig>,
    /// Validators stashes source for the chain.
    #[serde(default)]
    pub validators_source: Option<Source>,
    /// Collators for the system chains.
    /// TODO: To be supported in the future.
    #[serde(default)]
    pub collators: Vec<NodeConfig>,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainConfig {
    pub fn new() -> Self {
        Self {
            rpc_url: String::new(),
            signer: None,
            validators: Vec::new(),
            validators_source: None,
            collators: Vec::new(),
        }
    }

    pub fn with_validators(validators: Vec<NodeConfig>, validators_source: Option<Source>) -> Self {
        Self {
            rpc_url: String::new(),
            signer: None,
            validators,
            validators_source,
            collators: Vec::new(),
        }
    }

    pub fn signer_path(&self) -> Option<String> {
        self.signer
            .as_ref()
            .map(|s| s.path().to_string_lossy().into_owned())
    }

    pub fn signer_account_id(&self) -> Result<AccountId32, Error> {
        self.signer
            .as_ref()
            .ok_or(Error::SignerNotDefined)?
            .account_id()
    }

    pub fn is_qrcode_enabled(&self) -> bool {
        self.signer
            .as_ref()
            .map(|s| s.is_qrcode_enabled())
            .unwrap_or(false)
    }

    pub fn set_signer_path(&mut self, path: &str) {
        let signer = Signer::from_file(path).ok();
        self.signer = signer;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Host(SocketAddr);

impl Host {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self(SocketAddr::new(ip, port))
    }

    pub fn new_with_port(port: u16) -> Self {
        Self(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
    }

    pub fn http_url(&self) -> String {
        format!("http://{}", self.0)
    }

    pub fn ws_url(&self) -> String {
        format!("ws://{}", self.0)
    }

    pub fn host(&self, masked: bool) -> String {
        let ip = self.0.ip();

        if masked {
            match ip {
                IpAddr::V4(_) => "X.X.X.X".to_string(),
                IpAddr::V6(_) => "X:X:X:X:X:X:X:X".to_string(),
            }
        } else {
            ip.to_string()
        }
    }

    pub fn ip(&self) -> std::net::IpAddr {
        self.0.ip()
    }

    pub fn port(&self) -> u16 {
        self.0.port()
    }

    pub fn into(&self) -> SocketAddr {
        self.0
    }

    pub fn as_tuple(&self) -> (IpAddr, u16) {
        (self.0.ip(), self.0.port())
    }
}

impl Default for Host {
    fn default() -> Self {
        Self(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9944))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeConfig {
    Address(Stash),
    Detailed {
        stash: Stash,
        #[serde(default)]
        host_rpc: Host,
        #[serde(default)]
        ssh: Option<SshConfig>,
        #[serde(default)]
        commands: Option<Vec<CustomCommand>>,
    },
}

impl FromStr for NodeConfig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<AccountId32>()
            .map(NodeConfig::Address)
            .map_err(|e| Error::InvalidAddress(e.to_string()))
    }
}

fn default_enable() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    #[serde(default = "default_enable")]
    enable_validators: bool,
    #[serde(default)]
    enable_collators: bool,
    #[serde(default)]
    enable_rpcs: bool,
    #[serde(default = "default_enable")]
    enable_check_update: bool,
    #[serde(default = "default_enable")]
    show_nominators_counter: bool,
    #[serde(default = "default_enable")]
    show_nominators_stake: bool,
    #[serde(default = "default_enable")]
    show_own_stake: bool,
    #[serde(default)]
    show_commission: bool,
    #[serde(default = "default_enable")]
    show_payee: bool,
    #[serde(default = "default_enable")]
    show_keys: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            enable_validators: true,
            enable_collators: false,
            enable_rpcs: false,
            enable_check_update: true,
            show_nominators_counter: true,
            show_nominators_stake: true,
            show_own_stake: true,
            show_commission: false,
            show_payee: true,
            show_keys: true,
        }
    }
}

impl Features {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validators_enabled(&self) -> bool {
        self.enable_validators
    }

    pub fn collators_enabled(&self) -> bool {
        self.enable_collators
    }

    pub fn rpcs_enabled(&self) -> bool {
        self.enable_rpcs
    }

    pub fn check_update_enabled(&self) -> bool {
        self.enable_check_update
    }

    pub fn nominators_counter_visible(&self) -> bool {
        self.show_nominators_counter
    }

    pub fn nominators_stake_visible(&self) -> bool {
        self.show_nominators_stake
    }

    pub fn own_stake_visible(&self) -> bool {
        self.show_own_stake
    }

    pub fn commission_visible(&self) -> bool {
        self.show_commission
    }

    pub fn payee_visible(&self) -> bool {
        self.show_payee
    }

    pub fn keys_visible(&self) -> bool {
        self.show_keys
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
            return Ok(Self::default());
        }

        let mut config: Config = serde_yaml::from_str(&content)?;

        // Verify and validate if signer path exists
        if let Some(signer) = config.signer {
            config.signer = if signer.is_proxy_account_setup() {
                Some(signer)
            } else {
                Signer::from_file(signer.path()).ok()
            };
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

        // Validate custom commands don't clash with built-in call names
        for chain in &self.chains {
            for chain_config in chain.values() {
                for node in chain_config.validators.iter() {
                    if let NodeConfig::Detailed {
                        commands: Some(commands),
                        ..
                    } = node
                    {
                        for command in commands {
                            command.validate()?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn signer_path(&self) -> Option<String> {
        self.signer
            .as_ref()
            .map(|s| s.path().to_string_lossy().into_owned())
    }

    pub fn signer_account_id(&self) -> Result<AccountId32, Error> {
        self.signer
            .as_ref()
            .ok_or(Error::SignerNotDefined)?
            .account_id()
    }

    pub fn is_qrcode_enabled(&self) -> bool {
        self.signer
            .as_ref()
            .map(|s| s.is_qrcode_enabled())
            .unwrap_or(false)
    }

    pub fn set_signer_path(&mut self, path: &str) {
        let signer = Signer::from_file(path).ok();
        self.signer = signer;
    }

    pub fn set_signer_account(&mut self, address: &str) {
        let signer = Signer::from_address(address).ok();
        self.signer = signer;
    }

    pub fn set_vault_path(&mut self, path: &str) {
        let vault = Vault::from_file(path).ok();
        self.vault = vault;
    }

    pub fn explorer_url(&self, chain: &str, block_hash: &str) -> Option<String> {
        self.explorer.url.as_ref().map(|url| {
            url.replace("{chain}", chain)
                .replace("{block_hash}", block_hash)
        })
    }

    pub fn subcommand(&self) -> Option<&Subcommand> {
        self.subcommand.as_ref()
    }

    pub fn theme(&self) -> &Theme {
        self.themes.theme()
    }

    pub fn chains(&self) -> &Chains {
        &self.chains
    }

    pub fn features(&self) -> &Features {
        &self.features
    }

    fn set_default_theme(&mut self) {
        self.themes.active = default_active_theme();
    }

    pub fn logs_max_entries(&self) -> usize {
        self.logs.max_entries()
    }

    pub fn logs_file_path(&self) -> Option<&str> {
        self.logs.file_path()
    }
}

/// Provides default value for the configuration file path
fn default_config_path() -> &'static str {
    ".config.yaml"
}

/// Builds the CLI application
fn build_cli() -> clap::Command {
    let default_config_path = default_config_path();
    let default_proxy_path = default_proxy_path();

    clap::Command::new("suno")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            clap::Command::new("update")
                .about("Update SUNO latest or specified version for your host platform")
                .arg(
                    clap::Arg::new("version")
                        .index(1)
                        .value_name("VERSION")
                        .help("If given a version argument then `update` command updates the specified version, otherwise installs latest."),
                ),
        )
        .subcommand(network_subcommand(
            "polkadot",
            "Launch SUNO in light client mode on Polkadot",
        ))
        .subcommand(network_subcommand(
            "kusama",
            "Launch SUNO in light client mode on Kusama",
        ))
        .subcommand(network_subcommand(
            "paseo",
            "Launch SUNO in light client mode on Paseo",
        ))
        .subcommand(network_subcommand(
            "westend",
            "Launch SUNO in light client mode on Westend",
        ))
        .arg(
            clap::Arg::new("config-path")
                .short('c')
                .long("config-path")
                .value_name("FILE")
                .default_value(default_config_path)
                .help("Sets a custom config file path."),
        )
        .arg(
            clap::Arg::new("proxy-account")
                .short('p')
                .long("proxy-account")
                .value_name("PROXY_ADDRESS")
                .help("Sets a global proxy account used by Polkadot Vault."),
        )
        .arg(
            clap::Arg::new("proxy-path")
                .long("proxy-path")
                .value_name("FILE")
                .default_value(default_proxy_path)
                .help("Sets a global proxy account file path."),
        )
        .arg(
            clap::Arg::new("vault-secret-path")
                .long("vault-secret-path")
                .value_name("FILE")
                .help("Sets a global vault secret file path."),
        )
}

/// Returns a subcommand for a specific network (e.g. `polkadot`, `kusama`).
fn network_subcommand(name: &'static str, about: &'static str) -> clap::Command {
    clap::Command::new(name)
        .about(about)
        .arg(
            clap::Arg::new("validators")
                .long("validators")
                .short('v')
                .value_delimiter(',')
                .num_args(1..)
                .value_name("STASH_ADDRESS")
                .help(
                    "Validator stash addresses. \
                     Specify one or multiple addresses (e.g. stash_1,stash_2,stash_3).",
                ),
        )
        .arg(
            clap::Arg::new("validators-url")
                .long("validators-url")
                .short('u')
                .value_name("URL")
                .help("Validator source URL for stash addresses."),
        )
        .arg(
            clap::Arg::new("pat")
                .long("pat")
                .value_name("PAT")
                .help("Personal Access Token for the validators source URL."),
        )
        .arg(
            clap::Arg::new("proxy-account")
                .long("proxy-account")
                .short('p')
                .value_name("PROXY_ADDRESS")
                .help("Proxy account used by Polkadot Vault."),
        )
}

/// Validates a semantic version string (e.g. `1.2.3`, `v1.2.3`, `1.2.3-beta.1`).
/// Returns the normalized version without the `v` prefix on success.
fn validate_semver(input: &str) -> Result<Version, Error> {
    Version::parse(input.trim().trim_start_matches('v')).map_err(Error::InvalidVersion)
}

/// Loads the configuration from default paths and CLI arguments explicitly
/// overriding the default values.
fn get_config() -> Result<Config, Error> {
    let default_config_path = default_config_path();

    let matches = build_cli().get_matches();

    let config_path = matches
        .get_one::<String>("config-path")
        .map(|s| s.as_str())
        .unwrap_or(default_config_path);

    // Read and parse the config file
    let mut config = Config::from_file(config_path)?;

    // If not specified in the config file, load the signer proxy path
    // from the CLI, otherwise try to load the default
    if config.signer.is_none() {
        if let Some(path) = matches.get_one::<String>("proxy-path") {
            config.set_signer_path(path.as_str());
        }

        if let Some(account) = matches.get_one::<String>("proxy-account") {
            config.set_signer_account(account.as_str());
        }
    }

    // If not specified in the config file, load the vault private key path
    // from the CLI.
    if config.vault.is_none() {
        if let Some(path) = matches.get_one::<String>("vault-secret-path") {
            config.set_vault_path(path.as_str());
        }
    }

    // Process the subcommand
    let subcommand = match matches.subcommand() {
        Some(("update", sub)) => {
            let version = sub
                .get_one::<String>("version")
                .map(|s| validate_semver(s).map(|v| v.to_string()))
                .transpose()?;

            Some(Subcommand::Update { version })
        }
        Some((relay_chain, sub)) => {
            let runtime = SupportedRuntime::from_str(relay_chain)?;

            let validators = sub
                .get_many::<String>("validators")
                .unwrap_or_default()
                .map(|s| s.parse::<NodeConfig>())
                .collect::<Result<Vec<_>, _>>()?;

            let url = sub
                .get_one::<String>("validators-url")
                .cloned()
                .unwrap_or_default();

            let pat = sub.get_one::<String>("pat").cloned();

            let validators_source = if url.is_empty() {
                None
            } else {
                Some(Source::with(url, pat))
            };

            let rc_config = ChainConfig::with_validators(validators, validators_source);

            let mut chains = vec![HashMap::from([(runtime, rc_config)])];
            for system_chain in runtime.default_system_chains() {
                chains.push(HashMap::from([(system_chain, ChainConfig::new())]));
            }

            // Replace the chains in the config with the new chains
            config.chains = chains;

            let signer = sub
                .get_one::<String>("proxy-account")
                .map(|s| Signer::from_address(s))
                .transpose()?;

            // Replace the signer in the config with the new signer
            config.signer = signer;
            None
        }
        _ => None,
    };

    // Add the subcommand to the config
    config.subcommand = subcommand;

    // Validate the theme and set the default if necessary
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
    use crate::custom::{CommandKind, CustomCalls};
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
    fn test_detailed_with_host_and_commands() {
        let yaml = r#"
            stash: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            host_rpc: 192.0.2.100:9944
            commands:
              - name: Ping
                cmd: ping
                run: "echo 'Ping'"

              - name: Rotate and Set keys
                uses: calls/rotate_and_set_keys
        "#;
        let config: NodeConfig = serde_yaml::from_str(yaml).unwrap();

        match config {
            NodeConfig::Detailed {
                stash,
                host_rpc,
                commands,
                ..
            } => {
                assert_eq!(
                    stash.to_string(),
                    "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
                );
                assert_eq!(host_rpc.http_url(), "http://192.0.2.100:9944");
                let commands = commands.unwrap();
                assert_eq!(commands[0].name, "Ping");
                match &commands[0].kind {
                    CommandKind::Shell { cmd, run } => {
                        assert_eq!(cmd.as_deref(), Some("ping"));
                        assert_eq!(run, "echo 'Ping'");
                    }
                    _ => panic!("Expected Shell command"),
                }
                assert_eq!(commands[1].name, "Rotate and Set keys");
                match &commands[1].kind {
                    CommandKind::Uses(cmd) => {
                        assert!(matches!(cmd, CustomCalls::RotateAndSetKeys));
                    }
                    _ => panic!("Expected Uses command"),
                }
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
            NodeConfig::Detailed {
                stash, commands, ..
            } => {
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
            subcommand: None,
            chains: vec![],
            features: Features::default(),
            signer: None,
            vault: None,
            explorer: Explorer::default(),
            themes: Themes::default(),
            logs: Logs::default(),
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

    #[test]
    fn test_config_global_proxy_account_enables_qrcode() {
        let yaml = r#"
chains:
  - polkadot:
      rpc_url: "wss://rpc.polkadot.io"
      light_client: false
      validators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
features:
    enable_validators: true
    enable_collators: true
    enable_rpcs: false
signer:
  proxy_account: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
explorer:
    url: "https://polkadot.js.org/apps/?rpc=wss://{chain}.rpc.turboflakes.io#/explorer/query/{block_hash}"
"#;
        let file = create_temp_file(yaml);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.chains.len(), 1);
        assert!(config.is_qrcode_enabled());
    }

    #[test]
    fn test_config_chain_proxy_account_enables_qrcode() {
        let yaml = r#"
chains:
  - polkadot:
      rpc_url: "wss://rpc.polkadot.io"
      signer:
        proxy_account: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
      validators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
features:
    enable_validators: true
explorer:
    url: "https://polkadot.js.org/apps/?rpc=wss://{chain}.rpc.turboflakes.io#/explorer/query/{block_hash}"
"#;
        let file = create_temp_file(yaml);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.chains.len(), 1);
        let chain_config = config.chains[0]
            .get(&SupportedRuntime::Polkadot)
            .expect("polkadot chain should be present");

        assert_eq!(
            chain_config.signer_account_id().unwrap().to_string(),
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        );
        assert!(
            chain_config.is_qrcode_enabled(),
            "proxy_account should enable qrcode signing"
        );
    }

    #[test]
    fn test_valid_semver() {
        assert!(validate_semver("1.2.3").is_ok());
        assert!(validate_semver("v1.2.3").is_ok());
        assert!(validate_semver("1.2.3-beta.1").is_ok());
        assert!(validate_semver("1.2.3-alpha+build.42").is_ok());
    }

    #[test]
    fn test_invalid_semver() {
        assert!(validate_semver("").is_err());
        assert!(validate_semver("1.2").is_err());
        assert!(validate_semver("1.2.3.4").is_err());
        assert!(validate_semver("1.x.3").is_err());
        assert!(validate_semver("01.2.3").is_err());
    }

    // Helper function to create temporary file with content
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }
}
