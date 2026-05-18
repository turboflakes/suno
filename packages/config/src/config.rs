use crate::error::Error;
use crate::runtime::SupportedRuntime;
use crate::themes::{default_active_theme, Themes};
use lazy_static::lazy_static;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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

/// Built-in call methods derived from `Call`'s Display implementation.
/// Must be kept in sync with `suno_primitives::call::Call`.
const BUILTIN_CALL_NAMES: &[&str] = &[
    "bond",
    "bond_extra",
    "unbond",
    "rebond",
    "withdraw_unbonded",
    "set_payee",
    "validate",
    "chill",
    "set_keys",
    "purge_keys",
    "set_keys_async",
    "purge_keys_async",
    "rotate_and_set_keys", // CustomCalls::RotateAndSetKeys
];

/// Placeholders resolved automatically from the validator context, not from user input.
const BUILTIN_PLACEHOLDERS: &[&str] = &["stash"];

type Stash = AccountId32;

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
    #[serde(default)]
    pub light_client: bool,
    #[serde(default)]
    pub validators: Vec<NodeConfig>,
    #[serde(default)]
    pub collators: Vec<NodeConfig>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomCommand {
    pub name: String,
    #[serde(flatten)]
    pub kind: CommandKind,
}

impl std::fmt::Display for CustomCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            CommandKind::Shell { .. } => write!(f, "{}", self.name.as_str()),
            CommandKind::Uses(custom) => write!(f, "{}", custom.description()),
        }
    }
}

impl CustomCommand {
    pub fn cmd(&self) -> String {
        match &self.kind {
            CommandKind::Shell { cmd: None, .. } => {
                self.name.trim().to_lowercase().replace(" ", "_")
            }
            _ => self.kind.to_string(),
        }
    }

    pub fn is_shell(&self) -> bool {
        matches!(self.kind, CommandKind::Shell { .. })
    }

    pub fn is_super(&self) -> bool {
        matches!(self.kind, CommandKind::Uses(..))
    }

    /// Command name without `{...}` placeholders.
    /// e.g. "/echo {msg}" -> "msg"
    pub fn base_cmd(&self) -> String {
        self.cmd()
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// Display label with `{arg}` placeholders shown as `<arg>` for the UI hint
    /// e.g. "/echo {msg}" -> "/echo <msg>"
    pub fn placeholder(&self) -> String {
        self.cmd().replace("{", "<").replace("}", ">")
    }

    /// Unique placeholder names in `run`, excluding built-ins like `{stash}`.
    /// run = "echo {msg}" -> ["msg"]
    pub fn args(&self) -> Vec<String> {
        let CommandKind::Shell { run, .. } = &self.kind else {
            return vec![];
        };
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        let mut s = run.as_str();
        while let Some(open) = s.find('{') {
            s = &s[open + 1..];
            if let Some(close) = s.find('}') {
                let name = &s[..close];
                if !name.is_empty()
                    && !BUILTIN_PLACEHOLDERS.contains(&name)
                    && seen.insert(name.to_string())
                {
                    result.push(name.to_string());
                }
                s = &s[close + 1..];
            }
        }
        result
    }

    /// Returns a clone with `{name}` placeholders substituted positionally.
    /// Also substitutes in `cmd` so the display label reflects the resolved value.
    pub fn with_args(&self, values: &[&str]) -> Self {
        let mut cloned = self.clone();
        if let CommandKind::Shell { run, cmd } = &mut cloned.kind {
            for (name, value) in self.args().iter().zip(values) {
                let placeholder = format!("{{{}}}", name);
                *run = run.replace(&placeholder, value);
                if let Some(c) = cmd.as_mut() {
                    *c = c.replace(&placeholder, value);
                }
            }
        }
        cloned
    }

    pub fn validate(&self) -> Result<(), Error> {
        if let CommandKind::Shell { .. } = &self.kind {
            let cmd = self.base_cmd();
            if BUILTIN_CALL_NAMES.contains(&cmd.as_str()) {
                return Err(Error::InvalidCommand(cmd));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandKind {
    /// Built-in command: "calls/rotate_and_set_keys"
    Uses(CustomCalls),
    /// Shell command: "echo test"
    #[serde(untagged)]
    Shell {
        #[serde(default)]
        cmd: Option<String>, // shown in UI; falls back to `name` if absent
        run: String, // command to be executed: "echo test", "systemctl restart ..."
    },
}

impl std::fmt::Display for CommandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shell { cmd, .. } => {
                let display = cmd
                    .as_deref()
                    .and_then(|s| s.strip_prefix('/'))
                    .unwrap_or("ND");
                write!(f, "{}", display)
            }
            Self::Uses(call) => write!(f, "{}", call),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomCalls {
    #[serde(rename = "calls/rotate_and_set_keys")]
    RotateAndSetKeys,
}

impl CustomCalls {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RotateAndSetKeys => "rotate_and_set_keys",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::RotateAndSetKeys => {
                "Execute RPC call 'author_rotateKeysWithOwner' and Set sesion keys"
            }
        }
    }
}

impl std::fmt::Display for CustomCalls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Default SSH PORT
fn default_ssh_port() -> u16 {
    22
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshConfig {
    pub host: String,
    pub user: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    #[serde(default)]
    pub identity: Option<String>, // path to private key, None = use SSH agent
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    #[serde(default)]
    pub enable_validators: bool,
    #[serde(default)]
    pub enable_collators: bool,
    #[serde(default)]
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

/// Provides default value for the proxy account file path
fn default_proxy_path() -> String {
    ".proxy_account.json".to_string()
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

/// Provides default value for the configuration file path
fn default_config_path() -> &'static str {
    ".config.yaml"
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
    fn test_detailed_with_host_and_commands() {
        let yaml = r#"
            stash: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            host_rpc: 10.10.10.1:9944
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
                assert_eq!(host_rpc.http_url(), "http://10.10.10.1:9944");
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
