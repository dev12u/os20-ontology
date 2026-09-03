//! Predicate runtime: ontology metadata + registered Rust handlers.
//!
//! No arbitrary scripts. Flow/Connect/… stay Rust handlers; ontology types
//! constrain endpoints.

#![allow(clippy::result_large_err)]

use serde::{Deserialize, Serialize};

use crate::diagnostics::{DiagnosticCode, OntologyDiagnostic, RelatedLocation};
use crate::identity::{PredicateId, SemanticDomainId, TypeId};
use crate::relations::{RelationEdge, RelationKind};
use crate::snapshot::OntologySnapshot;

/// Declared endpoint constraints for a predicate / relation kind.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredicateRuntimeSpec {
    /// Predicate or relation name.
    pub id: String,
    /// Source must be assignable to this type (if set).
    pub source_type: Option<TypeId>,
    /// Target must be assignable to this type (if set).
    pub target_type: Option<TypeId>,
    /// Semantic domain.
    pub domain: Option<SemanticDomainId>,
}

impl OntologySnapshot {
    /// Predicates whose relation domain matches `domain`.
    pub fn predicates_in_domain(&self, domain: &SemanticDomainId) -> Vec<PredicateId> {
        self.predicates()
            .filter(|p| p.relation.domains.iter().any(|d| d == domain))
            .map(|p| p.id.clone())
            .collect()
    }

    /// Validate a core relation using ontology endpoint rules.
    pub fn validate_relation(&self, edge: &RelationEdge) -> Result<(), OntologyDiagnostic> {
        match edge.kind {
            RelationKind::Flow => validate_flow(self, edge),
            RelationKind::Connect | RelationKind::Allocate => {
                require_assignable_endpoints(self, edge, None, None)
            }
            _ => Ok(()),
        }
    }

    /// Ontology-defined predicate occurrence.
    pub fn validate_predicate(
        &self,
        spec: &PredicateRuntimeSpec,
        source: &TypeId,
        target: &TypeId,
    ) -> Result<(), OntologyDiagnostic> {
        if let Some(st) = &spec.source_type {
            self.check_assignable(source, st)
                .map_err(|_| illegal_endpoint("source", source, st))?;
        }
        if let Some(tt) = &spec.target_type {
            self.check_assignable(target, tt)
                .map_err(|_| illegal_endpoint("target", target, tt))?;
        }
        Ok(())
    }
}

fn validate_flow(snap: &OntologySnapshot, edge: &RelationEdge) -> Result<(), OntologyDiagnostic> {
    let physical = TypeId::new("@os20/core#PhysicalThing")
        .map_err(|_| OntologyDiagnostic::unresolved_type("@os20/core#PhysicalThing"))?;
    if snap.quantity_of(&edge.source).is_some() || snap.quantity_of(&edge.target).is_some() {
        return Err(illegal_endpoint("flow", &edge.source, &edge.target));
    }
    snap.check_assignable(&edge.source, &physical)
        .map_err(|_| illegal_endpoint("flow source", &edge.source, &physical))?;
    snap.check_assignable(&edge.target, &physical)
        .map_err(|_| illegal_endpoint("flow target", &edge.target, &physical))?;
    Ok(())
}

fn require_assignable_endpoints(
    snap: &OntologySnapshot,
    edge: &RelationEdge,
    src: Option<&TypeId>,
    tgt: Option<&TypeId>,
) -> Result<(), OntologyDiagnostic> {
    if let Some(s) = src {
        snap.check_assignable(&edge.source, s)?;
    }
    if let Some(t) = tgt {
        snap.check_assignable(&edge.target, t)?;
    }
    Ok(())
}

fn illegal_endpoint(role: &str, source: &TypeId, expected: &TypeId) -> OntologyDiagnostic {
    OntologyDiagnostic::new(
        DiagnosticCode::IllegalRelationEndpoint,
        format!(
            "illegal {role} endpoint `{}` (expected assignable to `{}`)",
            source.as_str(),
            expected.as_str()
        ),
    )
    .with_related(vec![RelatedLocation {
        role: role.into(),
        element: source.as_element().clone(),
        source_file: None,
    }])
}
