use super::config::CONFIG;
use super::errors::SnopsError;
use regex::Regex;
use std::str::FromStr;

use subxt_signer::{sr25519::Keypair, SecretUri};

/// Helper function to generate a keypair from the content of the seed file
pub fn get_keypair_from_seed_file() -> Result<Keypair, SnopsError> {
    let config = CONFIG.clone();

    // load data from seed file
    let data = std::fs::read_to_string(config.signer_path())?;

    // clear control characters from data
    let re = Regex::new(r"[\x00-\x1F]").unwrap();
    let data = re.replace_all(&data.trim(), "");

    // parse data into a secret
    let suri = SecretUri::from_str(&data)?;
    let keypair = Keypair::from_uri(&suri)?;
    Ok(keypair)
}
