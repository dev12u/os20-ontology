//! Packages, releases, roles, and provenance. Identity is not duplicated.

use serde::{Deserialize, Serialize};

use crate::fingerprint::ContentDigest;
use crate::identity::{ElementId, PackageId};

/// Resolver package role. Ontology packages reuse [`PackageRole::Ontology`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PackageRole {
    /// Workspace member.
    Workspace,
    /// Ordinary dependency.
    Dependency,
    /// Ontology package (existing resolver role).
    Ontology,
    /// OMG/KerML language standard library (not an OS20 ontology package).
    StandardLibrary,
}

/// Git-backed immutable package release coordinates. Not a semantic element id.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageReleaseRef {
    /// Package identity (`@os20/core`).
    pub package: PackageId,
    /// Published version (SemVer string as resolved).
    pub version: String,
    /// Repository URL (may be redacted by product surfaces).
    pub repository: Option<String>,
    /// Package root inside the tree.
    pub package_root: String,
    /// Git commit (hex).
    pub commit: Option<String>,
    /// Git tree (hex).
    pub tree: Option<String>,
    /// Manifest digest.
    pub manifest_digest: Option<ContentDigest>,
}

impl PackageReleaseRef {
    /// Bootstrap / in-memory release (no Git object).
    pub fn bootstrap(package: PackageId, version: &str) -> Self {
        Self {
            package,
            version: version.to_owned(),
            repository: None,
            package_root: "ontology".to_owned(),
            commit: None,
            tree: None,
            manifest_digest: None,
        }
    }
}

/// Source span in a package-relative file. Not identity.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan {
    /// Byte offset start.
    pub start: u32,
    /// Byte offset end.
    pub end: u32,
}

/// Where a declaration was authored. File move changes this, not named identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    /// Declaring package.
    pub package: PackageId,
    /// Package-relative source path (never an absolute machine path).
    pub source_file: String,
    /// Source span.
    pub span: SourceSpan,
    /// Commit that supplied the definition.
    pub commit: Option<String>,
    /// Tree that supplied the definition.
    pub tree: Option<String>,
}

impl Provenance {
    /// Catalog provenance (bootstrap files).
    pub fn catalog(package: PackageId, source_file: &str) -> Self {
        Self {
            package,
            source_file: source_file.to_owned(),
            span: SourceSpan { start: 0, end: 0 },
            commit: None,
            tree: None,
        }
    }
}

/// Visibility of an ontology contract member.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    /// Public contract (SemVer-relevant).
    #[default]
    Public,
    /// Private / package-local.
    Private,
}

/// Element lifecycle. Distinct from package release status.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum LifecycleStatus {
    /// Current definition.
    #[default]
    Active,
    /// Still resolvable; replacement is guidance only.
    Deprecated {
        /// Replacement semantic id, if declared.
        replacement: Option<ElementId>,
    },
}

/// How this snapshot was produced.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OntologySourceKind {
    /// `MemoryOntology::core()` bootstrap. Migration path to package-backed.
    Bootstrap,
    /// Locked ontology packages from the existing resolver/lock.
    PackageBacked,
}

/// Value origins. Kept distinct: default ≠ declared ≠ override ≠ calculated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ValueOrigin {
    /// Ontology default.
    Default,
    /// Explicitly declared on the element.
    Declared,
    /// Redefining feature override.
    Override,
    /// Derived/calculated (not stored as a second graph).
    Calculated,
}

/// Binding of a type reference after language binding.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TypeRef {
    /// Bound to a stable type id.
    Bound {
        /// Resolved type.
        id: crate::identity::TypeId,
    },
    /// Unresolved; preserved. Never silently replaced with a universal Any.
    Unresolved {
        /// Written name.
        written: String,
    },
}
