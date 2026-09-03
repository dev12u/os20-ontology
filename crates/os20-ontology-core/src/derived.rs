//! Rebuildable derived ontology index. SQLite row ids are never semantic identity.
//!
//! Persistence is a projection of [`crate::snapshot::OntologySnapshot`]. Deleting
//! `.os20` and reindexing from the same snapshot must yield the same fingerprints.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::fingerprint::ContentDigest;
use crate::identity::TypeId;
use crate::snapshot::OntologySnapshot;
use thiserror::Error;

/// Errors writing/reading a derived index.
#[derive(Debug, Error)]
pub enum DerivedIndexError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Derived indexes keyed by named identities (TEXT), never INTEGER PRIMARY KEY.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DerivedOntologyIndex {
    /// Runtime format that produced this index (must match
    /// [`crate::ONTOLOGY_RUNTIME_FORMAT_VERSION`] or the file is discarded).
    #[serde(default)]
    pub runtime_format_version: u32,
    /// Semantic fingerprint of the snapshot that produced this index.
    pub semantic_fingerprint: ContentDigest,
    /// Source fingerprint.
    pub source_fingerprint: ContentDigest,
    /// Type id → qualified name.
    pub type_by_id: BTreeMap<String, String>,
    /// Qualified name → type id.
    pub type_by_qualified_name: BTreeMap<String, String>,
    /// Outgoing specialize.
    pub specialize_out: BTreeMap<String, Vec<String>>,
    /// Incoming specialize.
    pub specialize_in: BTreeMap<String, Vec<String>>,
    /// Declaring type → property ids.
    pub properties_by_type: BTreeMap<String, Vec<String>>,
    /// Package → element ids.
    pub elements_by_package: BTreeMap<String, Vec<String>>,
}

impl DerivedOntologyIndex {
    /// Build from a snapshot (resolver/cache; not canonical).
    pub fn from_snapshot(snapshot: &OntologySnapshot) -> Self {
        let mut type_by_id = BTreeMap::new();
        let mut type_by_qualified_name = BTreeMap::new();
        let mut specialize_out = BTreeMap::new();
        let mut specialize_in = BTreeMap::new();
        let mut properties_by_type = BTreeMap::new();
        let mut elements_by_package: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for summary in snapshot.find_types("") {
            let id = summary.id.as_str().to_owned();
            type_by_id.insert(id.clone(), summary.id.local_name().to_owned());
            type_by_qualified_name.insert(id.clone(), id.clone());
            specialize_out.insert(
                id.clone(),
                summary
                    .supertypes
                    .iter()
                    .map(|t| t.as_str().to_owned())
                    .collect(),
            );
            specialize_in.insert(
                id.clone(),
                snapshot
                    .direct_subtypes(&summary.id)
                    .iter()
                    .map(|t| t.as_str().to_owned())
                    .collect(),
            );
            properties_by_type.insert(
                id.clone(),
                summary
                    .properties
                    .iter()
                    .map(|p| p.as_str().to_owned())
                    .collect(),
            );
            elements_by_package
                .entry(summary.package.as_str().to_owned())
                .or_default()
                .push(id);
        }
        for v in elements_by_package.values_mut() {
            v.sort();
        }
        Self {
            runtime_format_version: crate::versions::ONTOLOGY_RUNTIME_FORMAT_VERSION,
            semantic_fingerprint: snapshot.semantic_fingerprint().clone(),
            source_fingerprint: snapshot.source_fingerprint().clone(),
            type_by_id,
            type_by_qualified_name,
            specialize_out,
            specialize_in,
            properties_by_type,
            elements_by_package,
        }
    }

    /// Write under `{root}/ontology-index.json` (stands in for rebuildable SQLite).
    pub fn write_to_os20_dir(&self, os20_dir: &Path) -> Result<(), DerivedIndexError> {
        fs::create_dir_all(os20_dir)?;
        let path = os20_dir.join("ontology-index.json");
        fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    /// Read a derived index.
    pub fn read_from_os20_dir(os20_dir: &Path) -> Result<Self, DerivedIndexError> {
        let bytes = fs::read(os20_dir.join("ontology-index.json"))?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Type lookup via derived index (named ids only).
    pub fn lookup_type(&self, id: &TypeId) -> Option<&str> {
        self.type_by_id.get(id.as_str()).map(String::as_str)
    }

    /// False when the on-disk projection was written by a different runtime format.
    pub fn format_compatible(&self) -> bool {
        self.runtime_format_version == crate::versions::ONTOLOGY_RUNTIME_FORMAT_VERSION
    }
}

/// Delete `.os20` and rebuild from the same snapshot.
pub fn reindex(
    snapshot: &OntologySnapshot,
    os20_dir: &Path,
) -> Result<DerivedOntologyIndex, DerivedIndexError> {
    if os20_dir.exists() {
        fs::remove_dir_all(os20_dir)?;
    }
    let idx = DerivedOntologyIndex::from_snapshot(snapshot);
    idx.write_to_os20_dir(os20_dir)?;
    Ok(idx)
}
