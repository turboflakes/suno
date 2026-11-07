use crate::access::SshConfig;
use crate::custom::CustomCommand;
use crate::error::Error;
use crate::features::{Feature, Features, ALL_FEATURES};
use crate::logs::Logs;
use crate::prefs::{PinnedNode, Prefs};
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
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use subxt::utils::AccountId32;
use suno_theme::Theme;
use tracing::warn;

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
    /// Path the config was loaded from, used to locate the sibling prefs
    /// file that persists runtime preferences (e.g. the active theme).
    #[serde(skip)]
    config_path: PathBuf,
    /// Network launched from the CLI (e.g. `suno polkadot`), added as a suffix to the prefs
    /// file name so each network keeps its own runtime preferences.
    #[serde(skip)]
    network: Option<SupportedRuntime>,
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
            config_path: PathBuf::new(),
            network: None,
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
    /// TODO: deprecate signer in favor of signers
    #[serde(default)]
    pub signer: Option<Signer>,
    /// A chain can have more than one proxy account, e.g. one for its validators and another for its collators.
    #[serde(default)]
    signers: RwLock<Vec<Signer>>,
    /// Validators stashes for the chain.
    #[serde(default)]
    pub validators: Vec<NodeConfig>,
    /// Validators stashes source for the chain.
    #[serde(default)]
    pub validators_source: Option<Source>,
    /// Collators for the system chains.
    #[serde(default)]
    pub collators: Vec<NodeConfig>,
    /// Nominators stashes, only available on asset hub chains.
    #[serde(default)]
    pub nominators: Vec<NodeConfig>,
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
            signers: RwLock::new(Vec::new()),
            validators: Vec::new(),
            validators_source: None,
            collators: Vec::new(),
            nominators: Vec::new(),
        }
    }

    pub fn with_validators(validators: Vec<NodeConfig>, validators_source: Option<Source>) -> Self {
        Self {
            rpc_url: String::new(),
            signer: None,
            signers: RwLock::new(Vec::new()),
            validators,
            validators_source,
            collators: Vec::new(),
            nominators: Vec::new(),
        }
    }

    pub fn signer_path(&self) -> Option<String> {
        self.signer
            .as_ref()
            .map(|s| s.path().to_string_lossy().into_owned())
    }

    /// The accounts of the signers of the chain, the `signer` first.
    pub fn signer_account_ids(&self) -> Vec<AccountId32> {
        let mut accounts = Vec::new();
        if let Some(Ok(account)) = self.signer.as_ref().map(|signer| signer.account_id()) {
            accounts.push(account);
        }
        for signer in self.signers.read().unwrap().iter() {
            if let Ok(account) = signer.account_id() {
                if !accounts.contains(&account) {
                    accounts.push(account);
                }
            }
        }
        accounts
    }

    pub fn signer_account_id(&self) -> Result<AccountId32, Error> {
        self.signer_account_ids()
            .into_iter()
            .next()
            .ok_or(Error::SignerNotDefined)
    }

    pub fn has_signers(&self) -> bool {
        self.signer.is_some() || !self.signers.read().unwrap().is_empty()
    }

    /// Adds the proxy `account` as a signer of the chain, unless it already is one.
    pub fn add_signer(&self, account: AccountId32) {
        if self.signer_account_ids().contains(&account) {
            return;
        }
        if let Ok(signer) = Signer::from_address(&account.to_string()) {
            self.signers.write().unwrap().push(signer);
        }
    }

    /// Removes the proxy `account` from the signers added to the chain, not from the `signer` of
    /// the config file. Returns false when it wasn't one of them.
    pub fn remove_signer(&self, account: &AccountId32) -> bool {
        let mut signers = self.signers.write().unwrap();
        let count = signers.len();
        signers.retain(|signer| signer.account_id().ok().as_ref() != Some(account));
        signers.len() != count
    }

    pub fn is_qrcode_enabled(&self) -> bool {
        self.signer
            .as_ref()
            .map(|s| s.is_qrcode_enabled())
            .unwrap_or(false)
            || self
                .signers
                .read()
                .unwrap()
                .iter()
                .any(|s| s.is_qrcode_enabled())
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

impl NodeConfig {
    pub fn stash(&self) -> Stash {
        match self {
            Self::Address(stash) | Self::Detailed { stash, .. } => *stash,
        }
    }
}

impl FromStr for NodeConfig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<AccountId32>()
            .map(NodeConfig::Address)
            .map_err(|e| Error::InvalidAddress(e.to_string()))
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Explorer {
    url: Option<String>,
}

/// Writes `content` to `path`, readable and writable by the owner only (`0600` on unix).
pub(crate) fn write_private_atomic(path: &Path, content: &[u8]) -> Result<(), Error> {
    use std::io::Write;
    let tmp_name = format!(
        "{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or(".file")
    );
    let tmp_path = path.with_file_name(tmp_name);
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&tmp_path)?;
    file.write_all(content)?;
    file.sync_all()?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Self::from_file_for(path, None)
    }

    /// Loads the config as `from_file` does, for the `network` launched from the CLI, if any.
    /// The network is part of the prefs file name, so the preferences restored are the ones
    /// persisted by the same network.
    pub fn from_file_for<P: AsRef<Path>>(
        path: P,
        network: Option<SupportedRuntime>,
    ) -> Result<Self, Error> {
        let path = path.as_ref();

        let mut config = if !path.exists() {
            warn!(
                "Config file not found at {}, using defaults",
                path.display()
            );
            Self::default()
        } else {
            let content = fs::read_to_string(path)?;
            if content.is_empty() {
                Self::default()
            } else {
                serde_yaml::from_str(&content)?
            }
        };

        config.config_path = path.to_path_buf();
        config.network = network;

        // Verify and validate if signer path exists
        if let Some(signer) = config.signer {
            config.signer = if signer.is_proxy_account_setup() {
                Some(signer)
            } else {
                Signer::from_file(signer.path()).ok()
            };
        }

        // Load themes from the config file
        let themes = Themes::load(&config.themes.path)?;
        config.themes.set_themes(themes);

        // Apply preferences from the TUI, if any
        config.apply_prefs();

        Ok(config)
    }

    /// Path of the small sibling file used to persist runtime preferences
    /// across restarts, without touching the user's own config file. When launched for a
    /// network, e.g. `suno polkadot`, the network is the suffix: `.config.local.polkadot.yaml`.
    fn prefs_path(&self) -> PathBuf {
        let dir = self.config_path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = match self.network {
            Some(network) => format!(".config.local.{}.yaml", network.to_string().to_lowercase()),
            None => ".config.local.yaml".to_string(),
        };
        dir.join(file_name)
    }

    /// Path of the file where the smoldot database of a relay chain is persisted, e.g.
    /// `.smol.polkadot.db`, next to the config file.
    pub fn lightclient_db_path(&self, chain_name: &str) -> PathBuf {
        let dir = self.config_path.parent().unwrap_or_else(|| Path::new("."));
        dir.join(format!(".smol.{}.db", chain_name.to_lowercase()))
    }

    /// Reads the prefs file, if it exists and is well-formed. Defaults to
    /// an empty `Prefs` otherwise, so a missing or corrupt file never
    /// blocks startup.
    fn read_prefs(&self) -> Prefs {
        fs::read_to_string(self.prefs_path())
            .ok()
            .and_then(|content| serde_yaml::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// The validators pinned from the TUI, as saved in the prefs file.
    pub fn pinned_validators(&self) -> Vec<PinnedNode> {
        self.read_prefs().pinned_validators()
    }

    /// The collators pinned from the TUI, as saved in the prefs file.
    pub fn pinned_collators(&self) -> Vec<PinnedNode> {
        self.read_prefs().pinned_collators()
    }

    /// The features as set in the config file, before any change from the TUI. The file is read
    /// again, defaults apply if it is missing or malformed.
    fn file_features(&self) -> Features {
        let Ok(content) = fs::read_to_string(&self.config_path) else {
            return Features::default();
        };
        match serde_yaml::from_str::<Config>(&content) {
            Ok(config) => config.features,
            Err(_) => Features::default(),
        }
    }

    /// Every feature with the value set from the TUI, if any, otherwise the one of the config
    /// file.
    pub fn feature_states(&self) -> Vec<FeatureState> {
        let overrides = self.read_prefs().features;
        let file = self.file_features();
        ALL_FEATURES
            .into_iter()
            .map(|feature| {
                let saved = overrides.get(feature.key()).copied();
                FeatureState {
                    feature,
                    value: saved.unwrap_or_else(|| file.get(feature)),
                    changed: saved.is_some_and(|value| value != file.get(feature)),
                    running: self.features.get(feature),
                }
            })
            .collect()
    }

    /// Applies what was changed from the TUI, over what the config file sets: the last theme
    /// selected, if any, and the features.
    fn apply_prefs(&mut self) {
        if let Some(active) = self.load_prefs_active_theme() {
            self.themes.active = active;
        }
        self.load_prefs_features();
    }

    /// Reads and applies the features changed from the TUI, over the ones of the config file.
    fn load_prefs_features(&self) {
        for (key, value) in self.read_prefs().features {
            if let Some(feature) = Feature::from_key(&key) {
                self.features.set(feature, value);
            }
        }
    }

    /// Reads the persisted active theme name, if the prefs file exists,
    /// is well-formed, and names a theme that's still in the catalog.
    fn load_prefs_active_theme(&self) -> Option<String> {
        let active = self.read_prefs().active_theme?;
        if self.themes.catalog().iter().any(|(n, _)| n == &active) {
            Some(active)
        } else {
            None
        }
    }

    /// Persists the prefs file, overwriting it with the given snapshot.
    fn write_prefs(&self, prefs: &Prefs) -> Result<(), Error> {
        let content = serde_yaml::to_string(prefs)?;
        write_private_atomic(&self.prefs_path(), content.as_bytes())
    }

    /// The nominators are only available on the Asset Hub.
    fn validate_nominators(&self) -> Result<(), Error> {
        for chain in &self.chains {
            for (runtime, chain_config) in chain {
                if !chain_config.nominators.is_empty() && !runtime.is_asset_hub() {
                    return Err(Error::NominatorsNotSupported(runtime.to_string()));
                }
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Error> {
        // Validate that at least one chain is configured
        if self.chains.is_empty() {
            return Err(Error::ChainNotAvailable);
        }

        // Validate themes
        self.themes.validate()?;

        // Validate nominators
        self.validate_nominators()?;

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

    pub fn theme(&self) -> Theme {
        self.themes.theme()
    }

    /// Returns the available themes ordered by name, useful to loop through them.
    pub fn theme_catalog(&self) -> Vec<(String, Theme)> {
        self.themes.catalog()
    }

    /// Returns the index of the active theme within the catalog.
    pub fn theme_active_index(&self) -> usize {
        self.themes.active_index()
    }

    pub fn chains(&self) -> &Chains {
        &self.chains
    }

    /// The configuration of the chain `runtime`, if it is launched.
    fn chain_config(&self, runtime: SupportedRuntime) -> Option<&ChainConfig> {
        self.chains.iter().find_map(|chain| chain.get(&runtime))
    }

    /// Adds the proxy `account` to the signers of the chain `runtime`, if it is launched.
    pub fn add_proxy_account(&self, runtime: SupportedRuntime, account: AccountId32) {
        if let Some(chain_config) = self.chain_config(runtime) {
            chain_config.add_signer(account);
        }
    }

    /// The proxy accounts of the chain `runtime` as saved in the prefs.
    pub fn pinned_proxy_accounts(&self, runtime: SupportedRuntime) -> Vec<AccountId32> {
        self.read_prefs()
            .proxy_accounts()
            .into_iter()
            .filter(|(chain, _)| *chain == runtime)
            .map(|(_, account)| account)
            .collect()
    }

    /// The network launched from the CLI, e.g. `suno polkadot`, if any.
    pub fn network(&self) -> Option<SupportedRuntime> {
        self.network
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

    // The network launched, if any, so the prefs persisted are the ones of that network
    let network = match matches.subcommand() {
        Some(("update", _)) | None => None,
        Some((relay_chain, _)) => SupportedRuntime::from_str(relay_chain).ok(),
    };

    // Read and parse the config file
    let mut config = Config::from_file_for(config_path, network)?;

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

    // Add the proxy accounts added from the TUI to the signers of the chains launched. A chain
    // that is not launched is skipped, since it would be a new chain to connect to.
    for (runtime, account) in config.read_prefs().proxy_accounts() {
        config.add_proxy_account(runtime, account);
    }

    // Validate the configuration
    config.validate()?;

    Ok(config)
}

/// A feature and its value, as listed in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureState {
    pub feature: Feature,
    /// The value saved, which is the one in use unless the feature needs a restart.
    pub value: bool,
    /// Whether the value differs from the config file.
    pub changed: bool,
    /// The value in use right now.
    pub running: bool,
}

impl FeatureState {
    /// Whether the value only applies once the app is started again.
    pub fn pending_restart(&self) -> bool {
        self.value != self.running
    }

    /// The symbol shown for the value: filled when the feature is on.
    pub fn symbol(&self) -> &'static str {
        if self.value {
            "●"
        } else {
            "○"
        }
    }

    /// What toggling the feature does, to show as a hint.
    pub fn toggle_label(&self) -> &'static str {
        if self.value {
            "disable"
        } else {
            "enable"
        }
    }

    /// The value in the config file.
    pub fn file_value(&self) -> bool {
        self.value != self.changed
    }
}

/// Sets a feature from the TUI and persists it, so it is restored on the next launch. The
/// features that need a restart are only persisted. A value equal to the config file's is not
/// kept. Failures are logged and otherwise ignored.
pub fn save_feature(feature: Feature, value: bool) {
    if !feature.needs_restart() {
        CONFIG.features.set(feature, value);
    }
    let mut prefs = CONFIG.read_prefs();
    if value == CONFIG.file_features().get(feature) {
        prefs.features.remove(feature.key());
    } else {
        prefs.features.insert(feature.key().to_string(), value);
    }
    if let Err(e) = CONFIG.write_prefs(&prefs) {
        warn!("Failed to persist the feature {}: {}", feature.key(), e);
    }
}

/// Adds the proxy account to the signers of the chain `runtime`, and persists it so it is added
/// on the next launches too. Failures are logged and otherwise ignored.
pub fn save_proxy_account(runtime: SupportedRuntime, account: AccountId32) {
    CONFIG.add_proxy_account(runtime, account);
    let mut prefs = CONFIG.read_prefs();
    prefs.add_proxy_account(runtime, account);
    if let Err(e) = CONFIG.write_prefs(&prefs) {
        warn!("Failed to persist the proxy account: {}", e);
    }
}

/// Removes the proxy account from the signers of the chain `runtime`, and from the prefs. It only
/// removes the accounts added from the TUI or listed under `signers`. Failures are logged and
/// otherwise ignored.
pub fn remove_proxy_account(runtime: SupportedRuntime, account: AccountId32) {
    if let Some(chain_config) = CONFIG.chain_config(runtime) {
        chain_config.remove_signer(&account);
    }
    let mut prefs = CONFIG.read_prefs();
    prefs.remove_proxy_account(runtime, account);
    if let Err(e) = CONFIG.write_prefs(&prefs) {
        warn!("Failed to persist the removed proxy account: {}", e);
    }
}

/// Persists the active theme so it's restored on the next launch.
/// Failures are logged and otherwise ignored, this is a nice-to-have.
pub fn save_active_theme(name: &str) {
    let mut prefs = CONFIG.read_prefs();
    prefs.active_theme = Some(name.to_string());
    if let Err(e) = CONFIG.write_prefs(&prefs) {
        warn!("Failed to persist active theme: {}", e);
    }
}

/// Persists the validators pinned from the TUI so they are restored on the next launch.
/// Failures are logged and otherwise ignored, this is a nice-to-have.
pub fn save_pinned_validators(pinned: Vec<PinnedNode>) {
    let mut prefs = CONFIG.read_prefs();
    prefs.set_pinned_validators(pinned);
    if let Err(e) = CONFIG.write_prefs(&prefs) {
        warn!("Failed to persist the pinned validators: {}", e);
    }
}

/// Persists the collators pinned from the TUI so they are restored on the next launch.
/// Failures are logged and otherwise ignored, this is a nice-to-have.
pub fn save_pinned_collators(pinned: Vec<PinnedNode>) {
    let mut prefs = CONFIG.read_prefs();
    prefs.set_pinned_collators(pinned);
    if let Err(e) = CONFIG.write_prefs(&prefs) {
        warn!("Failed to persist the pinned collators: {}", e);
    }
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
            config_path: PathBuf::new(),
            network: None,
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
        assert!(config.features.validators_enabled());
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
        assert!(config.features.validators_enabled());
        assert!(config.features.collators_enabled());
    }

    #[test]
    fn test_config_nominators_on_asset_hub() {
        let yaml = r#"
chains:
  - asset_hub_kusama:
      rpc_url: "wss://rpc.asset.hub"
      nominators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        - stash: "5GTD7ZeD823BjpmZBCSzBQp7cvHR1Gunq7oDkurZr9zUev2n"
"#;
        let file = create_temp_file(yaml);
        let config = Config::from_file(file.path()).unwrap();
        let chain = config.chains[0].values().next().unwrap();

        assert_eq!(chain.nominators.len(), 2);
        assert!(chain.validators.is_empty());
        assert!(config.validate_nominators().is_ok());
    }

    #[test]
    fn test_config_nominators_default_to_none() {
        let yaml = r#"
chains:
  - asset_hub_kusama:
      rpc_url: "wss://rpc.asset.hub"
"#;
        let file = create_temp_file(yaml);
        let config = Config::from_file(file.path()).unwrap();

        assert!(config.chains[0]
            .values()
            .next()
            .unwrap()
            .nominators
            .is_empty());
    }

    #[test]
    fn test_config_nominators_are_not_supported_outside_asset_hub() {
        for chain in ["polkadot", "kusama", "people_kusama"] {
            let yaml = format!(
                r#"
chains:
  - {chain}:
      rpc_url: "wss://rpc.example.io"
      nominators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
"#
            );
            let file = create_temp_file(&yaml);
            let config = Config::from_file(file.path()).unwrap();

            assert!(
                matches!(
                    config.validate_nominators(),
                    Err(Error::NominatorsNotSupported(_))
                ),
                "nominators should be rejected on {chain}"
            );
        }
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

    #[test]
    fn test_missing_config_file_falls_back_to_defaults() {
        let config = Config::from_file("/nonexistent/path/config.yaml").unwrap();
        assert!(config.chains.is_empty());
    }

    #[test]
    fn test_active_theme_persists_across_reload() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        std::fs::write(&config_path, "chains:\n  - polkadot:\n").unwrap();

        let config = Config::from_file(&config_path).unwrap();
        let catalog = config.themes.catalog();
        let other = catalog
            .iter()
            .find(|(name, _)| name != &config.themes.active)
            .map(|(name, _)| name.clone())
            .expect("at least two builtin themes");
        assert_ne!(other, config.themes.active);

        let mut prefs = config.read_prefs();
        prefs.active_theme = Some(other.clone());
        config.write_prefs(&prefs).unwrap();
        assert!(dir.path().join(".config.local.yaml").exists());

        let reloaded = Config::from_file(&config_path).unwrap();
        assert_eq!(reloaded.themes.active, other);
    }

    #[test]
    fn test_prefs_file_has_the_network_as_suffix() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        std::fs::write(&config_path, "chains:\n  - polkadot:\n").unwrap();

        let config = Config::from_file_for(&config_path, Some(SupportedRuntime::Polkadot)).unwrap();
        let mut prefs = config.read_prefs();
        prefs.active_theme = Some(config.themes.active.clone());
        config.write_prefs(&prefs).unwrap();

        assert!(dir.path().join(".config.local.polkadot.yaml").exists());
        assert!(!dir.path().join(".config.local.yaml").exists());
        assert!(!dir.path().join(".config.local.polkadot.yaml.tmp").exists());
    }

    #[test]
    fn test_pinned_nodes_persist_across_reload() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        std::fs::write(&config_path, "chains:\n  - polkadot:\n").unwrap();

        let config = Config::from_file(&config_path).unwrap();
        let stash = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
            .parse::<AccountId32>()
            .unwrap();
        let validators = vec![PinnedNode {
            chain: SupportedRuntime::Polkadot,
            stash,
        }];
        let collators = vec![PinnedNode {
            chain: SupportedRuntime::AssetHubPolkadot,
            stash,
        }];

        let mut prefs = config.read_prefs();
        prefs.set_pinned_validators(validators.clone());
        prefs.set_pinned_collators(collators.clone());
        config.write_prefs(&prefs).unwrap();

        let reloaded = Config::from_file(&config_path).unwrap();
        assert_eq!(reloaded.pinned_validators(), validators);
        assert_eq!(reloaded.pinned_collators(), collators);

        // Same shape as the chains of the config file, with the addresses of the network
        let content = std::fs::read_to_string(dir.path().join(".config.local.yaml")).unwrap();
        let value: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
        let address = value["chains"][0]["polkadot"]["validators"][0]
            .as_str()
            .unwrap();
        assert!(address.starts_with('1'), "{address}");
        assert!(value["chains"][1]["asset_hub_polkadot"]["collators"][0].is_string());

        // Changing the validators keeps the collators
        let mut prefs = reloaded.read_prefs();
        prefs.set_pinned_validators(Vec::new());
        reloaded.write_prefs(&prefs).unwrap();
        let reloaded = Config::from_file(&config_path).unwrap();
        assert!(reloaded.pinned_validators().is_empty());
        assert_eq!(reloaded.pinned_collators(), collators);
    }

    #[test]
    fn test_write_prefs_preserves_other_fields() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        std::fs::write(&config_path, "chains:\n  - polkadot:\n").unwrap();

        let config = Config::from_file(&config_path).unwrap();

        let mut prefs = config.read_prefs();
        prefs.active_theme = Some("Suno Light".to_string());
        config.write_prefs(&prefs).unwrap();

        // A second, unrelated write shouldn't need to know about
        // active_theme, and shouldn't lose it either.
        let prefs = config.read_prefs();
        config.write_prefs(&prefs).unwrap();

        let reloaded = config.read_prefs();
        assert_eq!(reloaded.active_theme.as_deref(), Some("Suno Light"));
    }

    #[test]
    fn test_unknown_persisted_theme_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        std::fs::write(&config_path, "chains:\n  - polkadot:\n").unwrap();
        std::fs::write(
            dir.path().join(".config.local.yaml"),
            "active_theme: Ghost\n",
        )
        .unwrap();

        let config = Config::from_file(&config_path).unwrap();
        assert_ne!(config.themes.active, "Ghost");
    }
}
