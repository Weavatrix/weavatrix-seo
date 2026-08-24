//! Stable non-cryptographic content identity.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

const OFFSET: u128 = 0x6c62_272e_07bb_0142_62b8_2175_6295_c58d;
const PRIME: u128 = 0x0000_0000_0100_0000_0000_0000_0000_013B;

/// 128-bit FNV-1a digest used for page and finding identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContentHash(u128);

impl ContentHash {
    /// Digests bytes into a stable identity.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        let mut hash = OFFSET;
        for byte in bytes {
            hash ^= u128::from(*byte);
            hash = hash.wrapping_mul(PRIME);
        }
        Self(hash)
    }

    /// Digests UTF-8 text.
    #[must_use]
    pub fn of_str(text: &str) -> Self {
        Self::of(text.as_bytes())
    }

    /// Lower-hex encoding, 32 characters.
    #[must_use]
    pub fn hex(self) -> String {
        format!("{:032x}", self.0)
    }

    /// First eight hex characters for fingerprints.
    #[must_use]
    pub fn short(self) -> String {
        format!("{:08x}", u32::try_from(self.0 >> 96).unwrap_or(0))
    }
}

impl Display for ContentHash {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{}", self.hex())
    }
}

#[cfg(test)]
mod tests {
    use super::ContentHash;

    #[test]
    fn identical_input_is_stable() {
        assert_eq!(ContentHash::of_str("hello"), ContentHash::of_str("hello"));
        assert_ne!(ContentHash::of_str("hello"), ContentHash::of_str("Hello"));
        assert_eq!(ContentHash::of_str("hello").hex().len(), 32);
    }
}
