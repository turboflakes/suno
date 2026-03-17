pub mod error;

use crate::error::Error;
use regex::Regex;
use serde_json::Value;
use std::str::FromStr;
use subxt::utils::AccountId32;
use subxt_signer::{polkadot_js_compat::decrypt_json, sr25519::Keypair, SecretUri};
use suno_config::CONFIG;

/// Helper function to generate a keypair from the content of the seed file
pub fn load_keypair(password: &str) -> Result<Keypair, Error> {
    load_keypair_from_json_file(password)
}

/// Helper function to generate a keypair from the content of an exported polkadot-JS account
fn load_keypair_from_json_file(password: &str) -> Result<Keypair, Error> {
    let config = CONFIG.clone();

    // Load the signer json path
    let signer_path = config
        .signer_path()
        .ok_or(Error::SignerPathNotFound("json".into()))?;

    // Read data from json file
    let raw_data = std::fs::read_to_string(signer_path)?;

    let a = decrypt_json(&raw_data, password)?;

    Ok(a)
}

pub fn get_address_from_json_file() -> Result<AccountId32, Error> {
    let config = CONFIG.clone();

    // Load the signer json path
    let signer_path = config
        .signer_path()
        .ok_or(Error::SignerPathNotFound("json".into()))?;

    // Read data from json file
    let raw_data = std::fs::read_to_string(signer_path)?;

    let json: Value = serde_json::from_str(&raw_data)
        .map_err(|err| Error::Other(format!("Failed to parse JSON: {}", err)))?;

    let address = json["address"]
        .as_str()
        .ok_or_else(|| Error::Other("json file does not contain public 'address'".to_string()))?;

    let account =
        AccountId32::from_str(address).map_err(|_| Error::InvalidAddress(address.to_string()))?;

    Ok(account)
}

/// Helper function to generate a keypair from the content of the seed file
fn _load_keypair_from_seed_file(password: Option<String>) -> Result<Keypair, Error> {
    let config = CONFIG.clone();

    // Load the signer seed path
    let signer_path = config
        .signer_path()
        .ok_or(Error::SignerPathNotFound("seed".into()))?;

    // Read data from seed file
    let raw_data = std::fs::read_to_string(signer_path)?;

    // Clean data - remove whitespace and control characters
    let re = Regex::new(r"[\x00-\x1F]").unwrap();
    let clean_data = re.replace_all(raw_data.trim(), "").to_string();

    // Construct the secret URI with optional password
    let uri = match password {
        Some(pwd) => format!("{}///{}", clean_data, pwd),
        None => clean_data,
    };
    let suri = SecretUri::from_str(&uri)?;

    // Parse into keypair
    let keypair = Keypair::from_uri(&suri)?;
    Ok(keypair)
}
