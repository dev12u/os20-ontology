//! Ontology evolution relations. Distinct from engineering Specialize.

use serde::{Deserialize, Serialize};

use crate::identity::{ElementId, TypeId};

/// Refactoring / supersession edges. These do **not** change historical packages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EvolutionKind {
    /// Successor concept (guidance).
    Supersedes,
    /// Explicit replacement pointer.
    ReplacedBy,
    /// Split into several classifiers.
    SplitInto,
    /// Merged into a single classifier.
    MergedInto,
}

impl EvolutionKind {
    /// Registry name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supersedes => "Supersedes",
            Self::ReplacedBy => "ReplacedBy",
            Self::SplitInto => "SplitInto",
            Self::MergedInto => "MergedInto",
        }
    }
}

/// One evolution fact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionEdge {
    /// Kind.
    pub kind: EvolutionKind,
    /// Historical concept.
    pub from: ElementId,
    /// New concept.
    pub to: ElementId,
}

/// Modeling convention for continuous refactoring placeholders.
///
/// `Unallocated<T>` is **not** a parser keyword. The convention is a classifier
/// `Unallocated{T}` that specializes `T` and `UnallocatedThing`. It is a real
/// classifier, never SQL NULL / missing / undefined.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnallocatedPattern {
    /// Type being refactored.
    pub of: TypeId,
    /// Placeholder classifier.
    pub unallocated: TypeId,
}

impl UnallocatedPattern {
    /// Local name `Unallocated{Local}` under the same package as `of`.
    pub fn conventional_name(of: &TypeId) -> String {
        format!("Unallocated{}", of.local_name())
    }
}

/// Absence kinds. Kept distinct from [`UnallocatedPattern`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AbsenceKind {
    /// No value provided (missing field).
    MissingValue,
    /// Language null / empty optional — not an ontology classifier.
    Null,
    /// Ontological absence (nothing exists).
    Nothing,
    /// Binding failed; type unknown — not Any.
    Undefined,
}

/// Documented split: Person → Individual | Organization | UnallocatedPerson.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefactoringMap {
    /// Legacy concept.
    pub from: TypeId,
    /// Replacement classifiers.
    pub into: Vec<TypeId>,
}
