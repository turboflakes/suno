use crate::error::Error;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use subxt::utils::AccountId32;

/// Provides default value for the proxy account file path
fn default_proxy_path() -> String {
    ".proxy_account.json".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signer {
    proxy_account: Option<AccountId32>,
    #[serde(default = "default_proxy_path")]
    proxy_path: String,
}

impl Signer {
    /// Returns the proxy account, using the cached value if already loaded,
    /// otherwise reading it from `proxy_path`.
    pub fn account_id(&self) -> Result<AccountId32, Error> {
        if let Some(account) = &self.proxy_account {
            return Ok(account.clone());
        }
        Self::parse_account_from_path(self.path())
    }

    pub fn uses_polkadot_vault(&self) -> bool {
        self.proxy_account.is_some()
    }

    pub fn path(&self) -> &Path {
        Path::new(&self.proxy_path)
    }

    /// Constructs a `Signer` by eagerly parsing the proxy account from the
    /// JSON file at `path`. Fails fast if the file is missing or malformed.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let account = Self::parse_account_from_path(path).map_err(|e| {
            warn!("Failed to load signer from {}: {}", path.display(), e);
            e
        })?;
        Ok(Signer {
            proxy_account: Some(account),
            proxy_path: path.to_string_lossy().into_owned(),
        })
    }

    /// Reads and parses an `AccountId32` from a JSON file at `path`.
    fn parse_account_from_path(path: &Path) -> Result<AccountId32, Error> {
        let content =
            fs::read_to_string(path).map_err(|_| Error::InvalidPath(path.display().to_string()))?;

        if content.is_empty() {
            return Err(Error::InvalidContent(path.display().to_string()));
        }

        let json: Value = serde_json::from_str(&content)
            .map_err(|err| Error::Other(format!("Failed to parse JSON: {}", err)))?;

        let address = json["address"].as_str().ok_or_else(|| {
            Error::Other("json file does not contain public 'address'".to_string())
        })?;

        AccountId32::from_str(address).map_err(|_| Error::InvalidAddress(address.to_string()))
    }
}
