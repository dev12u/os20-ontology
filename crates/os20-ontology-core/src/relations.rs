//! Normalized OS20 relations plus registered Rust handlers.
//!
//! Relation meaning is interpreted in Rust, never via SQL triggers or scripts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::identity::{PredicateId, TypeId};
use crate::snapshot::OntologySnapshot;

/// Frozen core relation kinds understood by the ontology runtime.
///
/// These match the existing OS20 normalized relation registry. Evolution edges
/// are [`crate::evolution::EvolutionKind`], not this enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelationKind {
    /// Specialization (subtyping). Reused; not a second inheritance engine.
    Specialize,
    /// Feature/property declaration.
    Declares,
    /// Mixin contribution. Mixin != specialization.
    Mixin,
    /// Additive, non-subtyping extend.
    Extend,
    /// Constrain/describe (Specify).
    Specify,
    /// Composition (not inheritance).
    Compose,
    /// Feature typing.
    FeatureTyping,
    /// Feature redefinition.
    Redefines,
    /// Subsetting.
    Subset,
    /// Connect.
    Connect,
    /// Reference.
    Reference,
    /// Flow.
    Flow,
    /// Allocate.
    Allocate,
    /// Succession.
    Succession,
    /// Verify.
    Verify,
    /// Validate.
    Validate,
    /// Depend.
    Depend,
}

impl RelationKind {
    /// All core kinds in canonical order.
    pub const ALL: &'static [RelationKind] = &[
        Self::Specialize,
        Self::Declares,
        Self::Mixin,
        Self::Extend,
        Self::Specify,
        Self::Compose,
        Self::FeatureTyping,
        Self::Redefines,
        Self::Subset,
        Self::Connect,
        Self::Reference,
        Self::Flow,
        Self::Allocate,
        Self::Succession,
        Self::Verify,
        Self::Validate,
        Self::Depend,
    ];

    /// Registry name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Specialize => "Specialize",
            Self::Declares => "Declares",
            Self::Mixin => "Mixin",
            Self::Extend => "Extend",
            Self::Specify => "Specify",
            Self::Compose => "Compose",
            Self::FeatureTyping => "FeatureTyping",
            Self::Redefines => "Redefines",
            Self::Subset => "Subset",
            Self::Connect => "Connect",
            Self::Reference => "Reference",
            Self::Flow => "Flow",
            Self::Allocate => "Allocate",
            Self::Succession => "Succession",
            Self::Verify => "Verify",
            Self::Validate => "Validate",
            Self::Depend => "Depend",
        }
    }

    /// Parse a registry name.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.as_str() == name)
    }
}

/// Cardinality when the language frontend supplied multiplicity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Multiplicity {
    /// Inclusive lower bound.
    pub lower: u32,
    /// Inclusive upper bound; `None` means unbounded (`*`).
    pub upper: Option<u32>,
}

/// Optional algebraic flags only when a handler implements them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationAlgebra {
    /// Symmetric relation.
    pub symmetric: bool,
    /// Transitive relation.
    pub transitive: bool,
}

/// Ontology-defined relation type (custom predicates).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationType {
    /// Predicate identity.
    pub id: PredicateId,
    /// Source classifier.
    pub source_type: TypeId,
    /// Target classifier.
    pub target_type: TypeId,
    /// Optional multiplicity.
    pub cardinality: Option<Multiplicity>,
    /// Inverse predicate, if declared.
    pub inverse: Option<PredicateId>,
    /// Algebra implemented by the Rust handler.
    pub algebra: RelationAlgebra,
    /// Semantic domains this predicate participates in.
    pub domains: Vec<crate::identity::SemanticDomainId>,
}

/// One occurrence of a relation in the ontology snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationEdge {
    /// Core kind or custom predicate name for dispatch.
    pub kind: RelationKind,
    /// Source type.
    pub source: TypeId,
    /// Target type or feature type.
    pub target: TypeId,
}

/// Interpretation produced by a Rust handler.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationInterpretation {
    /// Handler key.
    pub kind: String,
    /// Human-readable summary.
    pub summary: String,
}

/// `relation type → registered Rust handler`.
pub type RelationHandler = fn(&OntologySnapshot, &RelationEdge) -> RelationInterpretation;

/// Registry of core relation handlers.
#[derive(Clone, Debug)]
pub struct RelationRegistry {
    handlers: BTreeMap<RelationKind, RelationHandler>,
}

impl Default for RelationRegistry {
    fn default() -> Self {
        Self::core()
    }
}

impl RelationRegistry {
    /// Built-in handlers for the frozen relation catalog.
    pub fn core() -> Self {
        let mut handlers: BTreeMap<RelationKind, RelationHandler> = BTreeMap::new();
        for kind in RelationKind::ALL {
            handlers.insert(*kind, default_handler);
        }
        Self { handlers }
    }

    /// Look up a handler.
    pub fn handler(&self, kind: RelationKind) -> Option<RelationHandler> {
        self.handlers.get(&kind).copied()
    }

    /// Interpret an edge.
    pub fn interpret(
        &self,
        snapshot: &OntologySnapshot,
        edge: &RelationEdge,
    ) -> Option<RelationInterpretation> {
        self.handler(edge.kind).map(|h| h(snapshot, edge))
    }

    /// Ontology packages cannot register handlers; only this built-in set runs.
    pub fn is_builtin_only(&self) -> bool {
        self.handlers.len() == RelationKind::ALL.len()
            && RelationKind::ALL
                .iter()
                .all(|k| self.handlers.contains_key(k))
    }
}

fn default_handler(_snapshot: &OntologySnapshot, edge: &RelationEdge) -> RelationInterpretation {
    RelationInterpretation {
        kind: edge.kind.as_str().to_owned(),
        summary: format!(
            "{} {} → {}",
            edge.kind.as_str(),
            edge.source.as_str(),
            edge.target.as_str()
        ),
    }
}
