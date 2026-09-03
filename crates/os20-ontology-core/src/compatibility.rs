//! Precise type relations supplied to the existing SemanticResolver.
//!
//! This module does **not** replace merge ranking or the resolver. It answers
//! type questions from IDs and Specialize edges only (no name matching).

#![allow(clippy::result_large_err)]

use serde::{Deserialize, Serialize};

use crate::diagnostics::{DiagnosticCode, OntologyDiagnostic, RelatedLocation};
use crate::identity::{PropertyId, TypeId};
use crate::package::TypeRef;
use crate::snapshot::OntologySnapshot;

/// Quantity dimension token (`M`, `L`, `Θ`, …). Conversion is deferred.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QuantityDimension(String);

impl QuantityDimension {
    /// Construct from a canonical token.
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    /// Canonical token.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why assignability failed. Never uses type display names for the decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AssignabilityReject {
    /// One or both types are unknown in this snapshot.
    UnknownType,
    /// Not a Specialize descendant (and not the same type).
    NotSubtype,
    /// Both are quantities with different dimensions.
    QuantityDimensionMismatch {
        /// Value dimension.
        from: QuantityDimension,
        /// Target dimension.
        to: QuantityDimension,
    },
}

/// Result of [`OntologySnapshot::assignability`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Assignability {
    /// Identical TypeId.
    Same,
    /// Value specializes target.
    Subtype,
    /// Rejected.
    Rejected(AssignabilityReject),
}

impl Assignability {
    /// True when assignment is allowed.
    pub fn allowed(self) -> bool {
        !matches!(self, Self::Rejected(_))
    }
}

impl OntologySnapshot {
    /// Exact TypeId equality.
    pub fn is_same_type(&self, a: &TypeId, b: &TypeId) -> bool {
        a == b
    }

    /// `ty` is a supertype of `descendant` (or equal).
    pub fn is_supertype_of(&self, ty: &TypeId, descendant: &TypeId) -> bool {
        self.is_subtype_of(descendant, ty)
    }

    /// Unique least common supertype in the Specialize DAG, if well-defined.
    ///
    /// Does not linearize multiple inheritance. Returns `None` when there is no
    /// common ancestor or several incomparable minimal common ancestors.
    pub fn least_common_supertype(&self, a: &TypeId, b: &TypeId) -> Option<TypeId> {
        let a_set: Vec<TypeId> = self.supertypes(a);
        let b_set: Vec<TypeId> = self.supertypes(b);
        let common: Vec<TypeId> = a_set
            .iter()
            .filter(|t| b_set.contains(t))
            .cloned()
            .collect();
        if common.is_empty() {
            return None;
        }
        let mut minimal: Vec<TypeId> = common
            .iter()
            .filter(|c| {
                !common
                    .iter()
                    .any(|o| o != *c && self.is_subtype_of(o, c) && !self.is_same_type(o, c))
            })
            .cloned()
            .collect();
        minimal.sort();
        if minimal.len() == 1 {
            minimal.pop()
        } else {
            None
        }
    }

    /// Full assignability decision (quantities + classifiers).
    pub fn assignability(&self, value: &TypeId, target: &TypeId) -> Assignability {
        if self.is_same_type(value, target) {
            return Assignability::Same;
        }
        if self.type_decl(value).is_none() || self.type_decl(target).is_none() {
            return Assignability::Rejected(AssignabilityReject::UnknownType);
        }
        if let (Some(vq), Some(tq)) = (self.quantity_of(value), self.quantity_of(target)) {
            if vq.dimension != tq.dimension {
                return Assignability::Rejected(AssignabilityReject::QuantityDimensionMismatch {
                    from: QuantityDimension::new(&vq.dimension),
                    to: QuantityDimension::new(&tq.dimension),
                });
            }
        }
        if self.is_subtype_of(value, target) {
            Assignability::Subtype
        } else {
            Assignability::Rejected(AssignabilityReject::NotSubtype)
        }
    }

    /// Quantity record if this type (or a Specialize ancestor) is a quantity.
    pub fn quantity_of(&self, id: &TypeId) -> Option<&crate::model::QuantityType> {
        for anc in self.supertypes(id) {
            if let Some(q) = self.quantity(&anc) {
                return Some(q);
            }
        }
        None
    }

    /// KerML/SysML covariant redefinition: redefining type must specialize the
    /// original feature type (narrowing allowed, widening and dimension change not).
    pub fn redefinition_compatible(
        &self,
        original: &TypeId,
        redefining: &TypeId,
    ) -> Result<(), OntologyDiagnostic> {
        match self.assignability(redefining, original) {
            Assignability::Same | Assignability::Subtype => Ok(()),
            Assignability::Rejected(AssignabilityReject::QuantityDimensionMismatch {
                from,
                to,
            }) => Err(OntologyDiagnostic::new(
                DiagnosticCode::QuantityDimensionMismatch,
                format!(
                    "quantity dimension `{}` cannot be assigned to `{}`",
                    from.as_str(),
                    to.as_str()
                ),
            )
            .with_related(vec![
                loc("ontology definition", original),
                loc("offending declaration", redefining),
            ])),
            Assignability::Rejected(_) => Err(OntologyDiagnostic::new(
                DiagnosticCode::IncompatibleOverride,
                format!(
                    "redefining type `{}` does not specialize `{}` (KerML covariant redefinition)",
                    redefining.as_str(),
                    original.as_str()
                ),
            )
            .with_related(vec![
                loc("inherited contributor", original),
                loc("offending declaration", redefining),
            ])),
        }
    }

    /// Assignment check with diagnostic (no name guessing).
    pub fn check_assignable(
        &self,
        value: &TypeId,
        target: &TypeId,
    ) -> Result<(), OntologyDiagnostic> {
        match self.assignability(value, target) {
            Assignability::Same | Assignability::Subtype => Ok(()),
            Assignability::Rejected(AssignabilityReject::QuantityDimensionMismatch {
                from,
                to,
            }) => Err(OntologyDiagnostic::new(
                DiagnosticCode::QuantityDimensionMismatch,
                format!(
                    "cannot assign quantity `{}` ({}) to `{}` ({})",
                    value.as_str(),
                    from.as_str(),
                    target.as_str(),
                    to.as_str()
                ),
            )
            .with_related(vec![
                loc("ontology definition", target),
                loc("offending declaration", value),
            ])),
            Assignability::Rejected(AssignabilityReject::UnknownType) => {
                Err(OntologyDiagnostic::unresolved_type(&format!(
                    "{} / {}",
                    value.as_str(),
                    target.as_str()
                )))
            }
            Assignability::Rejected(_) => Err(OntologyDiagnostic::new(
                DiagnosticCode::IncompatibleAssignment,
                format!(
                    "`{}` is not assignable to `{}`",
                    value.as_str(),
                    target.as_str()
                ),
            )
            .with_related(vec![
                loc("ontology definition", target),
                loc("offending declaration", value),
            ])),
        }
    }

    /// Feature typing: bound TypeId only. Unresolved is not assignable.
    pub fn feature_type_id(&self, ty: &TypeRef) -> Option<TypeId> {
        match ty {
            TypeRef::Bound { id } => Some(id.clone()),
            TypeRef::Unresolved { .. } => None,
        }
    }

    /// Override of `inherited` by `local` for the same property identity.
    pub fn override_property(
        &self,
        property: &PropertyId,
        inherited_ty: &TypeId,
        local_ty: &TypeId,
    ) -> Result<(), OntologyDiagnostic> {
        let _ = property;
        self.redefinition_compatible(inherited_ty, local_ty)
    }
}

fn loc(role: &str, id: &TypeId) -> RelatedLocation {
    RelatedLocation {
        role: role.into(),
        element: id.as_element().clone(),
        source_file: None,
    }
}
