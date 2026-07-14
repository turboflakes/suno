pub mod error;

use crate::error::Error;
use regex::Regex;
use std::str::FromStr;
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
    let signer_path = config.signer_path().ok_or(Error::PathNotFound)?;

    // Read data from json file
    let raw_data = std::fs::read_to_string(signer_path)?;

    let a = decrypt_json(&raw_data, password)?;

    Ok(a)
}

/// Helper function to generate a keypair from the content of the seed file
fn _load_keypair_from_seed_file(password: Option<String>) -> Result<Keypair, Error> {
    let config = CONFIG.clone();

    // Load the signer seed path
    let signer_path = config.signer_path().ok_or(Error::PathNotFound)?;

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
