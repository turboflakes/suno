pub mod error;

use crate::error::Error;
use regex::Regex;
use std::str::FromStr;
use subxt_signer::{sr25519::Keypair, SecretUri};
use suno_config::CONFIG;

/// Helper function to generate a keypair from the content of the seed file
pub fn load_keypair(password: Option<String>) -> Result<Keypair, Error> {
    let config = CONFIG.clone();

    // Read data from seed file
    let raw_data = std::fs::read_to_string(config.signer_path())?;

    // Clean data - remove whitespace and control characters
    let re = Regex::new(r"[\x00-\x1F]").unwrap();
    let clean_data = re.replace_all(&raw_data.trim(), "").to_string();

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
