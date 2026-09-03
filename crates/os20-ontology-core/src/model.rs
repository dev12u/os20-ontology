//! Classifiers, features, predicates, domains, and quantities.

use serde::{Deserialize, Serialize};

use crate::identity::{PredicateId, PropertyId, SemanticDomainId, TypeId, UnitId};
use crate::package::{LifecycleStatus, Provenance, TypeRef, Visibility};
use crate::relations::Multiplicity;

/// KerML/SysML-aligned ontology type categories (not unrelated OOP).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OntologyTypeKind {
    /// Classifier (`Thing`, `PhysicalThing`, …).
    Classifier,
    /// Data type.
    DataType,
    /// Structure.
    Structure,
    /// Value type.
    ValueType,
    /// Enumeration.
    Enumeration,
    /// Physical quantity type.
    Quantity,
    /// Unit.
    Unit,
    /// Quantity dimension (M, L, T, …) — not a semantic domain.
    QuantityDimension,
    /// Predicate / relation type.
    Predicate,
    /// Explicit relation type classifier.
    RelationType,
}

/// Declaration of an ontology type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDecl {
    /// Stable type id.
    pub id: TypeId,
    /// Kind.
    pub kind: OntologyTypeKind,
    /// Direct Specialize targets (multiple specialization allowed).
    pub specializes: Vec<TypeId>,
    /// Mixin contributions (not ancestry).
    pub mixins: Vec<TypeId>,
    /// Semantic domains this type participates in (many allowed).
    pub domains: Vec<SemanticDomainId>,
    /// Declared features.
    pub properties: Vec<PropertyDecl>,
    /// Lifecycle.
    pub lifecycle: LifecycleStatus,
    /// Contract visibility.
    pub visibility: Visibility,
    /// Provenance (not identity).
    pub provenance: Provenance,
    /// Optional comment (source fingerprint only; excluded from semantic hash).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl TypeDecl {
    /// Canonical semantic payload (no comments, no absolute paths).
    pub fn semantic_canonical(&self) -> String {
        let mut specs: Vec<_> = self.specializes.iter().map(|t| t.as_str()).collect();
        specs.sort_unstable();
        let mut mix: Vec<_> = self.mixins.iter().map(|t| t.as_str()).collect();
        mix.sort_unstable();
        let mut domains: Vec<_> = self.domains.iter().map(|d| d.as_str()).collect();
        domains.sort_unstable();
        let mut props: Vec<_> = self
            .properties
            .iter()
            .map(PropertyDecl::semantic_canonical)
            .collect();
        props.sort();
        format!(
            "type\t{}\t{:?}\t{}\t{}\t{}\t{}\t{:?}\t{:?}",
            self.id.as_str(),
            self.kind,
            specs.join(","),
            mix.join(","),
            domains.join(","),
            props.join(";"),
            self.lifecycle,
            self.visibility
        )
    }
}

/// Ontology-declared feature. Type is a [`TypeId`] after binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyDecl {
    /// Property identity.
    pub id: PropertyId,
    /// Bound or unresolved type.
    pub ty: TypeRef,
    /// KerML/SysML multiplicity from the frontend.
    pub multiplicity: Option<Multiplicity>,
    /// Default value, if any (distinct from declared instance values).
    pub default_value: Option<String>,
    /// Visibility.
    pub visibility: Visibility,
}

impl PropertyDecl {
    /// Semantic canonical form.
    pub fn semantic_canonical(&self) -> String {
        let ty = match &self.ty {
            TypeRef::Bound { id } => id.as_str().to_owned(),
            TypeRef::Unresolved { written } => format!("unresolved:{written}"),
        };
        format!(
            "{}\t{}\t{:?}\t{:?}\t{:?}",
            self.id.as_str(),
            ty,
            self.multiplicity,
            self.default_value,
            self.visibility
        )
    }
}

/// Semantic domain (Design, Physics, …) — ontology-defined, not a package.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDomain {
    /// Domain id.
    pub id: SemanticDomainId,
    /// Display name.
    pub name: String,
    /// Declaring ontology package local name.
    pub defined_in: crate::identity::PackageId,
}

/// Physical quantity type (data structures only; no solver in Prompt 1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantityType {
    /// Quantity classifier.
    pub id: TypeId,
    /// Dimension token (`M`, `L`, `T`, `L/T^2`, …) — representable, not solved.
    pub dimension: String,
    /// Preferred unit, if any.
    pub preferred_unit: Option<UnitId>,
    /// Value type (typically Real).
    pub value_type: TypeId,
}

/// Named predicate applicable to ontology elements.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Predicate {
    /// Predicate id.
    pub id: PredicateId,
    /// Relation type payload.
    pub relation: crate::relations::RelationType,
}

/// Public type report. Contains no database identifiers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSummary {
    /// Type id.
    pub id: TypeId,
    /// Kind.
    pub kind: OntologyTypeKind,
    /// Package that defines it.
    pub package: crate::identity::PackageId,
    /// Ontology package version that supplied meaning.
    pub ontology_version: String,
    /// Source revision (commit) if package-backed.
    pub source_revision: Option<String>,
    /// Direct Specialize targets, sorted.
    pub supertypes: Vec<TypeId>,
    /// Mixin types, sorted. Not supertypes.
    pub mixins: Vec<TypeId>,
    /// Semantic domains, sorted.
    pub domains: Vec<SemanticDomainId>,
    /// Declared properties, sorted.
    pub properties: Vec<PropertyId>,
    /// Lifecycle.
    pub lifecycle: LifecycleStatus,
    /// Provenance.
    pub provenance: Provenance,
}

/// Effective property after Specialize/Declares inheritance (computed, not stored
/// as a second canonical graph).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveProperty {
    /// Property id.
    pub id: PropertyId,
    /// Declaring type.
    pub declared_by: TypeId,
    /// Type ref.
    pub ty: TypeRef,
    /// Origin of a default, if present.
    pub default_origin: Option<crate::package::ValueOrigin>,
}
