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

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SESSION_KEYS: &str = "0xe107640186c4cd86fe87b791c577207c5da12792485dec2fb266956094f059a36402031bfc45d8d60828e74bff23aa289616fb628cdc05c74f904de0754ba943128595e6edeaf17d3c0b30bad6ea7198f8ac524f710b9e17e71e72f0529a9c011e1b9724dfd5d2bd54dea69cb2ced1728e590f3e9a5860f9b2181c81884e88083276f1653b8b1db8491928f63bb41091bb33e1a93d2d8d9a0d86e7e034e23741035f56c8a251e0189bf636daace9671ebb6562976f054cd2252114f3e2b3cbc28e";

    #[test]
    fn test_keys_from_str_valid_with_0x_prefix() {
        let result = Keys::from_str(VALID_SESSION_KEYS);
        assert!(
            result.is_ok(),
            "Should parse valid session keys with 0x prefix"
        );

        let keys = result.unwrap();

        // Verify lengths
        assert_eq!(keys.grandpa_bytes.len(), 32);
        assert_eq!(keys.babe_bytes.len(), 32);
        assert_eq!(keys.para_validator_bytes.len(), 32);
        assert_eq!(keys.para_assignment_bytes.len(), 32);
        assert_eq!(keys.authority_discovery_bytes.len(), 32);
        assert_eq!(keys.beefy_bytes.len(), 33);

        // Verify first key (grandpa)
        let expected_grandpa =
            hex::decode("e107640186c4cd86fe87b791c577207c5da12792485dec2fb266956094f059a3")
                .unwrap();
        assert_eq!(keys.grandpa_bytes, expected_grandpa.as_slice());
    }

    #[test]
    fn test_keys_from_str_valid_without_0x_prefix() {
        let keys_without_prefix = VALID_SESSION_KEYS.trim_start_matches("0x");
        let result = Keys::from_str(keys_without_prefix);
        assert!(
            result.is_ok(),
            "Should parse valid session keys without 0x prefix"
        );
    }

    #[test]
    fn test_keys_from_str_all_keys_parsed_correctly() {
        let keys = Keys::from_str(VALID_SESSION_KEYS).unwrap();

        // Expected values for each key
        let expected_grandpa = "e107640186c4cd86fe87b791c577207c5da12792485dec2fb266956094f059a3";
        let expected_babe = "6402031bfc45d8d60828e74bff23aa289616fb628cdc05c74f904de0754ba943";
        let expected_para_validator =
            "128595e6edeaf17d3c0b30bad6ea7198f8ac524f710b9e17e71e72f0529a9c01";
        let expected_para_assignment =
            "1e1b9724dfd5d2bd54dea69cb2ced1728e590f3e9a5860f9b2181c81884e8808";
        let expected_authority_discovery =
            "3276f1653b8b1db8491928f63bb41091bb33e1a93d2d8d9a0d86e7e034e23741";
        let expected_beefy = "035f56c8a251e0189bf636daace9671ebb6562976f054cd2252114f3e2b3cbc28e";

        assert_eq!(hex::encode(keys.grandpa_bytes), expected_grandpa);
        assert_eq!(hex::encode(keys.babe_bytes), expected_babe);
        assert_eq!(
            hex::encode(keys.para_validator_bytes),
            expected_para_validator
        );
        assert_eq!(
            hex::encode(keys.para_assignment_bytes),
            expected_para_assignment
        );
        assert_eq!(
            hex::encode(keys.authority_discovery_bytes),
            expected_authority_discovery
        );
        assert_eq!(hex::encode(keys.beefy_bytes), expected_beefy);
    }

    #[test]
    fn test_keys_from_str_invalid_hex() {
        let invalid_hex = "0xZZZZ640186c4cd86fe87b791c577207c5da12792485dec2fb266956094f059a36402031bfc45d8d60828e74bff23aa289616fb628cdc05c74f904de0754ba943128595e6edeaf17d3c0b30bad6ea7198f8ac524f710b9e17e71e72f0529a9c011e1b9724dfd5d2bd54dea69cb2ced1728e590f3e9a5860f9b2181c81884e88083276f1653b8b1db8491928f63bb41091bb33e1a93d2d8d9a0d86e7e034e23741035f56c8a251e0189bf636daace9671ebb6562976f054cd2252114f3e2b3cbc28e";

        let result = Keys::from_str(invalid_hex);
        assert!(result.is_err(), "Should fail with invalid hex characters");

        match result.unwrap_err() {
            KeysError::InvalidHex(_) => {}
            _ => panic!("Expected InvalidHex error"),
        }
    }

    #[test]
    fn test_keys_from_str_too_short() {
        // Only 192 bytes instead of 193
        let too_short = "0xe107640186c4cd86fe87b791c577207c5da12792485dec2fb266956094f059a36402031bfc45d8d60828e74bff23aa289616fb628cdc05c74f904de0754ba943128595e6edeaf17d3c0b30bad6ea7198f8ac524f710b9e17e71e72f0529a9c011e1b9724dfd5d2bd54dea69cb2ced1728e590f3e9a5860f9b2181c81884e88083276f1653b8b1db8491928f63bb41091bb33e1a93d2d8d9a0d86e7e034e23741035f56c8a251e0189bf636daace9671ebb6562976f054cd2252114f3e2b3cbc2";

        let result = Keys::from_str(too_short);
        assert!(result.is_err(), "Should fail with incorrect length");

        match result.unwrap_err() {
            KeysError::InvalidHexLength(len) => {
                assert_eq!(len, 192, "Should report length of 192");
            }
            _ => panic!("Expected InvalidHexLength error"),
        }
    }

    #[test]
    fn test_keys_from_str_too_long() {
        // 194 bytes instead of 193
        let too_long = format!("{}ff", VALID_SESSION_KEYS);

        let result = Keys::from_str(&too_long);
        assert!(result.is_err(), "Should fail with incorrect length");

        match result.unwrap_err() {
            KeysError::InvalidHexLength(len) => {
                assert_eq!(len, 194, "Should report length of 194");
            }
            _ => panic!("Expected InvalidHexLength error"),
        }
    }

    #[test]
    fn test_keys_from_str_empty_string() {
        let result = Keys::from_str("");
        assert!(result.is_err(), "Should fail with empty string");

        match result.unwrap_err() {
            KeysError::InvalidHexLength(len) => {
                assert_eq!(len, 0, "Should report length of 0");
            }
            _ => panic!("Expected InvalidHexLength error"),
        }
    }

    #[test]
    fn test_keys_from_str_only_0x() {
        let result = Keys::from_str("0x");
        assert!(result.is_err(), "Should fail with only 0x prefix");

        match result.unwrap_err() {
            KeysError::InvalidHexLength(len) => {
                assert_eq!(len, 0, "Should report length of 0");
            }
            _ => panic!("Expected InvalidHexLength error"),
        }
    }

    #[test]
    fn test_keys_from_str_odd_length_hex() {
        // Hex string with odd number of characters (invalid)
        let odd_hex = "0xe107640186c4cd86fe87b791c577207c5da12792485dec2fb266956094f059a36402031bfc45d8d60828e74bff23aa289616fb628cdc05c74f904de0754ba943128595e6edeaf17d3c0b30bad6ea7198f8ac524f710b9e17e71e72f0529a9c011e1b9724dfd5d2bd54dea69cb2ced1728e590f3e9a5860f9b2181c81884e88083276f1653b8b1db8491928f63bb41091bb33e1a93d2d8d9a0d86e7e034e23741035f56c8a251e0189bf636daace9671ebb6562976f054cd2252114f3e2b3cbc28";

        let result = Keys::from_str(odd_hex);
        assert!(result.is_err(), "Should fail with odd-length hex string");
    }

    #[test]
    fn test_keys_roundtrip() {
        // Parse keys from string
        let keys = Keys::from_str(VALID_SESSION_KEYS).unwrap();

        // Convert back to hex string
        let hex_output = format!(
            "0x{}{}{}{}{}{}",
            hex::encode(keys.grandpa_bytes),
            hex::encode(keys.babe_bytes),
            hex::encode(keys.para_validator_bytes),
            hex::encode(keys.para_assignment_bytes),
            hex::encode(keys.authority_discovery_bytes),
            hex::encode(keys.beefy_bytes),
        );

        // Should match original
        assert_eq!(
            hex_output.to_lowercase(),
            VALID_SESSION_KEYS.to_lowercase(),
            "Roundtrip should produce same hex string"
        );
    }

    #[test]
    fn test_keys_clone_and_eq() {
        let keys1 = Keys::from_str(VALID_SESSION_KEYS).unwrap();
        let keys2 = keys1.clone();

        assert_eq!(keys1, keys2, "Cloned keys should be equal");
    }

    #[test]
    fn test_keys_debug() {
        let keys = Keys::from_str(VALID_SESSION_KEYS).unwrap();
        let debug_str = format!("{:?}", keys);

        assert!(
            debug_str.contains("Keys"),
            "Debug output should contain Keys"
        );
    }

    #[test]
    fn test_keys_display() {
        let keys = Keys::from_str(VALID_SESSION_KEYS).unwrap();
        let display_str = format!("{}", keys);

        // Should display as concatenated hex
        assert!(
            display_str.starts_with("0x"),
            "Display should start with 0x"
        );
        assert_eq!(
            display_str.len(),
            2 + (193 * 2), // "0x" + 193 bytes * 2 hex chars per byte
            "Display should have correct length"
        );
    }

    #[test]
    fn test_keys_error_display() {
        let err1 = KeysError::InvalidHexLength(100);
        assert_eq!(
            err1.to_string(),
            "Invalid hex length: expected 193 bytes, got 100"
        );

        let err2 = KeysError::Other("custom error".to_string());
        assert_eq!(err2.to_string(), "Other error: custom error");
    }
}
