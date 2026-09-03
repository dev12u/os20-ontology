//! Bound SourceGraph facts produced by frozen `os20-language`.
//!
//! Ontology compilation consumes these declarations, never lexer tokens.
//! KerML/SysML files remain Git authority; this graph is the binder output.

use serde::{Deserialize, Serialize};

use crate::identity::PackageId;
use crate::package::{LifecycleStatus, SourceSpan, Visibility};
use crate::relations::RelationKind;

/// Bound workspace graph (cross-file / cross-package ids already resolved when
/// `qualified` is set; otherwise the compiler applies import-scope binding).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundSourceGraph {
    /// Package that owns these facts.
    pub package: String,
    /// Package-relative source facts. Order is not semantic.
    pub elements: Vec<BoundElement>,
    /// Explicit relation edges (Specialize, Mixin, …).
    #[serde(default)]
    pub relations: Vec<BoundRelation>,
    /// File-level comments (source fingerprint only).
    #[serde(default)]
    pub file_comments: Vec<FileComment>,
}

/// Comment attached to a file, not an element.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileComment {
    /// Package-relative path.
    pub path: String,
    /// Comment text.
    pub text: String,
}

/// One bound named declaration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundElement {
    /// Local name (`Thing`).
    pub name: String,
    /// Fully bound id if the frontend already resolved it.
    #[serde(default)]
    pub qualified: Option<String>,
    /// Classifier / DataType / Structure / Quantity / Unit / Predicate / Domain.
    pub kind: String,
    /// Visibility from the language.
    #[serde(default)]
    pub visibility: Visibility,
    /// Written specialize targets (binder names or `@pkg#Name`).
    #[serde(default)]
    pub specializes: Vec<String>,
    /// Mixin names.
    #[serde(default)]
    pub mixins: Vec<String>,
    /// Features.
    #[serde(default)]
    pub features: Vec<BoundFeature>,
    /// Semantic domain local names or ids.
    #[serde(default)]
    pub domains: Vec<String>,
    /// Quantity dimension token (`M`, `L`, …) when kind is Quantity.
    #[serde(default)]
    pub quantity_dimension: Option<String>,
    /// Preferred unit local name.
    #[serde(default)]
    pub preferred_unit: Option<String>,
    /// Value type for quantities.
    #[serde(default)]
    pub value_type: Option<String>,
    /// Evolution replacement written name.
    #[serde(default)]
    pub replaced_by: Option<String>,
    /// Split-into written names.
    #[serde(default)]
    pub split_into: Vec<String>,
    /// Lifecycle.
    #[serde(default = "active_lifecycle")]
    pub lifecycle: LifecycleStatus,
    /// Comment (semantic no-op).
    #[serde(default)]
    pub comment: Option<String>,
    /// Package-relative file.
    pub file: String,
    /// Span in that file.
    #[serde(default)]
    pub span: SourceSpan,
}

fn active_lifecycle() -> LifecycleStatus {
    LifecycleStatus::Active
}

/// Bound feature / property.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundFeature {
    /// Feature name.
    pub name: String,
    /// Type written name or qualified id.
    pub ty: String,
    /// Optional default (distinct from instance values).
    #[serde(default)]
    pub default_value: Option<String>,
}

/// Bound relation occurrence.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundRelation {
    /// Relation kind name (`Specialize`, `Mixin`, …).
    pub kind: String,
    /// Source written name.
    pub source: String,
    /// Target written name.
    pub target: String,
}

impl BoundRelation {
    /// Parse kind against the frozen registry.
    pub fn relation_kind(&self) -> Option<RelationKind> {
        RelationKind::parse(&self.kind)
    }
}

/// Language-frontend gap: annotations not expressible in current KerML/SysML.
///
/// Stored as explicit metadata on the bound graph — not a hidden source
/// convention and not a second file syntax.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyMetadataExtension {
    /// Semantic domains when the language has no domain annotation.
    #[serde(default)]
    pub domains: Vec<String>,
    /// Quantity dimension when not a language construct.
    #[serde(default)]
    pub quantity_dimension: Option<String>,
}

/// Delta from an incremental SourceGraph rebuild.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceGraphDelta {
    /// Changed package-relative files.
    pub changed_files: Vec<String>,
    /// Removed files.
    pub removed_files: Vec<String>,
    /// Named definitions whose file path changed (identity preserved).
    pub moved: Vec<MovedDefinition>,
    /// True when only comments/formatting changed.
    pub comment_only: bool,
}

/// File move of a named definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovedDefinition {
    /// Semantic element id (unchanged).
    pub element: String,
    /// Previous path.
    pub from_file: String,
    /// New path.
    pub to_file: String,
}

/// Package manifest facts used to detect ontology role.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyManifest {
    /// `@scope/name`.
    pub name: PackageId,
    /// Version.
    pub version: String,
    /// `PackageRole` from `role = "ontology"` / `kind = "ontology"`.
    pub role: crate::package::PackageRole,
    /// Required dependencies.
    #[serde(default)]
    pub dependencies: Vec<PackageId>,
    /// Optional dependencies (only if the manifest expressed optional).
    #[serde(default)]
    pub optional_dependencies: Vec<PackageId>,
    /// Package root inside the Git tree.
    #[serde(default = "default_root")]
    pub package_root: String,
}

fn default_root() -> String {
    ".".into()
}
