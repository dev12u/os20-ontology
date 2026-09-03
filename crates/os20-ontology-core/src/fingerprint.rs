//! Deterministic ontology fingerprints. No SQLite ids, paths, or comment text.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hex SHA-256 digest of canonical bytes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentDigest(String);

impl ContentDigest {
    /// Hash arbitrary canonical bytes.
    pub fn hash_bytes(bytes: &[u8]) -> Self {
        let mut h = Sha256::new();
        h.update(bytes);
        Self(hex(h.finalize().as_slice()))
    }

    /// Hash sorted unique lines (order-independent). Unprefixed; prefer
    /// [`Self::hash_domain_sorted_lines`] for ontology fingerprints.
    pub fn hash_sorted_lines(lines: &mut Vec<String>) -> Self {
        Self::hash_domain_sorted_lines("ontology-generic-v1", lines)
    }

    /// Hash sorted unique lines under a semantic purpose prefix.
    ///
    /// Domains such as `ontology-semantic-v1` and `ontology-source-v1` must not
    /// share a raw SHA-256 namespace.
    pub fn hash_domain_sorted_lines(domain: &str, lines: &mut Vec<String>) -> Self {
        lines.sort();
        lines.dedup();
        let mut h = Sha256::new();
        h.update(domain.as_bytes());
        h.update([0]);
        for line in lines.iter() {
            h.update(line.as_bytes());
            h.update([0x0a]);
        }
        Self(hex(h.finalize().as_slice()))
    }

    /// Hex string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Wrap a hex string already produced by this hasher.
    pub(crate) fn from_hex_inner(hex: &str) -> Self {
        Self(hex.to_owned())
    }
}

impl std::fmt::Display for ContentDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Pair of source vs semantic fingerprints.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyFingerprints {
    /// Git/lock/manifest/source revision set (comments and file text can change this).
    pub source: ContentDigest,
    /// Semantic definitions only (comments excluded).
    pub semantic: ContentDigest,
}

fn hex(bytes: &[u8]) -> String {
    const T: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(T[(b >> 4) as usize] as char);
        out.push(T[(b & 0xf) as usize] as char);
    }
    out
}
