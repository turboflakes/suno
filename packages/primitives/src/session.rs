use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Keys {
    pub grandpa_bytes: [u8; 32],
    pub babe_bytes: [u8; 32],
    pub para_validator_bytes: [u8; 32],
    pub para_assignment_bytes: [u8; 32],
    pub authority_discovery_bytes: [u8; 32],
    #[serde(serialize_with = "serialize_beefy_bytes")]
    pub beefy_bytes: [u8; 33],
}

fn serialize_beefy_bytes<S>(bytes: &[u8; 33], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bytes(bytes)
}

impl Default for Keys {
    fn default() -> Self {
        Self {
            grandpa_bytes: [0u8; 32],
            babe_bytes: [0u8; 32],
            para_validator_bytes: [0u8; 32],
            para_assignment_bytes: [0u8; 32],
            authority_discovery_bytes: [0u8; 32],
            beefy_bytes: [0u8; 33],
        }
    }
}

impl std::fmt::Display for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{}{}{}{}{}{}",
            hex::encode(self.grandpa_bytes),
            hex::encode(self.babe_bytes),
            hex::encode(self.para_validator_bytes),
            hex::encode(self.para_assignment_bytes),
            hex::encode(self.authority_discovery_bytes),
            hex::encode(self.beefy_bytes),
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum KeysError {
    #[error("Invalid hex string: {0}")]
    InvalidHex(#[from] hex::FromHexError),
    #[error("Invalid hex length: expected 193 bytes, got {0}")]
    InvalidHexLength(usize),
    #[error("Other error: {0}")]
    Other(String),
}

impl FromStr for Keys {
    type Err = KeysError;
    fn from_str(keys: &str) -> Result<Self, Self::Err> {
        // Strip "0x" prefix if present
        let hex_str = keys.trim_start_matches("0x");

        // Decode hex to bytes
        let bytes = hex::decode(hex_str).map_err(|e| KeysError::InvalidHex(e))?;

        // Validate length: 32+32+32+32+32+33 = 193 bytes
        if bytes.len() != 193 {
            return Err(KeysError::InvalidHexLength(bytes.len()));
        }

        // Parse each key from the concatenated bytes
        let mut offset = 0;

        // Grandpa (32 bytes)
        let mut grandpa_bytes = [0u8; 32];
        grandpa_bytes.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Babe (32 bytes)
        let mut babe_bytes = [0u8; 32];
        babe_bytes.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Para Validator (32 bytes)
        let mut para_validator_bytes = [0u8; 32];
        para_validator_bytes.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Para Assignment (32 bytes)
        let mut para_assignment_bytes = [0u8; 32];
        para_assignment_bytes.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Authority Discovery (32 bytes)
        let mut authority_discovery_bytes = [0u8; 32];
        authority_discovery_bytes.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Beefy (33 bytes - ECDSA public key)
        let mut beefy_bytes = [0u8; 33];
        beefy_bytes.copy_from_slice(&bytes[offset..offset + 33]);

        Ok(Self {
            grandpa_bytes,
            babe_bytes,
            para_validator_bytes,
            para_assignment_bytes,
            authority_discovery_bytes,
            beefy_bytes,
        })
    }
}
