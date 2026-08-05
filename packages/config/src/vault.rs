use crate::error::Error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::str::FromStr;
use subxt_signer::{
    sr25519::{Keypair, PublicKey, Signature},
    SecretUri,
};

/// Represents a vault secret path. Used to sign metadata/chain-spec QRs that are to be loaded into Polkadot Vault.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Vault {
    #[serde(default)]
    secret_path: Option<String>,
}

impl Vault {
    /// Constructs a `Vault` by eagerly parsing the seed from the
    /// file at `path`. Fails fast if the file is missing or malformed.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let _ = Self::parse(path)?;
        Ok(Vault {
            secret_path: Some(path.to_string_lossy().into_owned()),
        })
    }

    pub fn sign(&self, payload: &[u8]) -> Result<(PublicKey, Signature), Error> {
        let path = self.secret_path.as_ref().ok_or(Error::NoKey)?;
        let suri = Self::parse(Path::new(path))?; // re-read + re-validate at use time
        let pair = Keypair::from_uri(&suri)?;
        Ok((pair.public_key(), pair.sign(payload)))
    }

    fn parse(path: &Path) -> Result<SecretUri, Error> {
        let content =
            fs::read_to_string(path).map_err(|_| Error::InvalidPath(path.display().to_string()))?;

        // Clean data - remove whitespace and control characters
        let re = Regex::new(r"[\x00-\x1F]").unwrap();
        let content_cleaned = re.replace_all(content.trim(), "").to_string();

        // Parse into secret URI
        let suri = SecretUri::from_str(&content_cleaned)?;
        Ok(suri)
    }
}
