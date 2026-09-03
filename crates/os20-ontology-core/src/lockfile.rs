//! `os20.lock` pins. There is no independent ontology lockfile.

use serde::{Deserialize, Serialize};

use crate::fingerprint::ContentDigest;
use crate::identity::{IdentityError, PackageId};
use crate::package::{PackageReleaseRef, PackageRole};

/// One locked package (existing lock format, ontology-aware).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Package name.
    pub name: String,
    /// Version.
    pub version: String,
    /// `path` / `git` / `registry`.
    #[serde(default)]
    pub source: String,
    /// Git repository.
    #[serde(default)]
    pub repository: Option<String>,
    /// Commit.
    #[serde(default)]
    pub commit: Option<String>,
    /// Tree.
    #[serde(default)]
    pub tree: Option<String>,
    /// Package root.
    #[serde(default)]
    pub package_root: Option<String>,
    /// Manifest digest hex.
    #[serde(default)]
    pub manifest_digest: Option<String>,
    /// Role when recorded.
    #[serde(default)]
    pub role: Option<String>,
    /// Dependencies as `name@version` strings.
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl LockedPackage {
    /// Parse package id.
    pub fn package_id(&self) -> Result<PackageId, IdentityError> {
        PackageId::new(&self.name)
    }

    /// Ontology role from lock or default dependency.
    pub fn package_role(&self) -> PackageRole {
        match self.role.as_deref() {
            Some("ontology") => PackageRole::Ontology,
            Some("standardLibrary") | Some("standard_library") => PackageRole::StandardLibrary,
            Some("workspace") => PackageRole::Workspace,
            _ => PackageRole::Dependency,
        }
    }

    /// Release coordinates.
    pub fn release_ref(&self) -> Result<PackageReleaseRef, IdentityError> {
        Ok(PackageReleaseRef {
            package: self.package_id()?,
            version: self.version.clone(),
            repository: self.repository.clone(),
            package_root: self.package_root.clone().unwrap_or_else(|| ".".to_owned()),
            commit: self.commit.clone(),
            tree: self.tree.clone(),
            manifest_digest: self.manifest_digest.as_deref().map(ContentDigest::from_hex),
        })
    }
}

/// Lockfile (`format = 1`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Os20Lockfile {
    /// Format version.
    #[serde(default = "one")]
    pub format: u32,
    /// Pins.
    #[serde(default)]
    pub package: Vec<LockedPackage>,
}

fn one() -> u32 {
    1
}

impl Os20Lockfile {
    /// Parse JSON lock (tests) or serde value. TOML-shaped JSON is accepted.
    pub fn from_json(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Ontology pins only.
    pub fn ontology_packages(&self) -> Vec<&LockedPackage> {
        self.package
            .iter()
            .filter(|p| p.package_role() == PackageRole::Ontology)
            .collect()
    }

    /// Find a pin.
    pub fn get(&self, name: &str) -> Option<&LockedPackage> {
        self.package.iter().find(|p| p.name == name)
    }
}

impl ContentDigest {
    /// Reconstruct from hex produced by this crate.
    pub fn from_hex(hex: &str) -> Self {
        Self::from_hex_inner(hex)
    }
}
