//! Bridge: OntologyRuntime supplies type knowledge to SemanticResolver.
//!
//! OS20 does **not** replace SemanticResolver. Callers keep using the resolver
//! for merge ranking, lock, and graph binding. This trait is the ontology
//! injection point.

use crate::fingerprint::ContentDigest;
use crate::identity::TypeId;
use crate::runtime::OntologyRuntime;
use crate::snapshot::OntologySnapshot;

/// Knowledge the existing resolver may query.
pub trait SemanticResolverOntology: Send + Sync {
    /// Same type.
    fn is_same_type(&self, a: &TypeId, b: &TypeId) -> bool;
    /// Subtype.
    fn is_subtype_of(&self, ty: &TypeId, ancestor: &TypeId) -> bool;
    /// Assignable.
    fn is_assignable_to(&self, value: &TypeId, target: &TypeId) -> bool;
    /// Ontology fingerprint mixed into resolved-element fingerprints.
    fn ontology_fingerprint(&self) -> &ContentDigest;
}

impl SemanticResolverOntology for OntologySnapshot {
    fn is_same_type(&self, a: &TypeId, b: &TypeId) -> bool {
        OntologySnapshot::is_same_type(self, a, b)
    }

    fn is_subtype_of(&self, ty: &TypeId, ancestor: &TypeId) -> bool {
        OntologySnapshot::is_subtype_of(self, ty, ancestor)
    }

    fn is_assignable_to(&self, value: &TypeId, target: &TypeId) -> bool {
        OntologySnapshot::is_assignable_to(self, value, target)
    }

    fn ontology_fingerprint(&self) -> &ContentDigest {
        self.semantic_fingerprint()
    }
}

impl SemanticResolverOntology for OntologyRuntime {
    fn is_same_type(&self, a: &TypeId, b: &TypeId) -> bool {
        self.snapshot().is_same_type(a, b)
    }

    fn is_subtype_of(&self, ty: &TypeId, ancestor: &TypeId) -> bool {
        self.snapshot().is_subtype_of(ty, ancestor)
    }

    fn is_assignable_to(&self, value: &TypeId, target: &TypeId) -> bool {
        self.snapshot().is_assignable_to(value, target)
    }

    fn ontology_fingerprint(&self) -> &ContentDigest {
        self.snapshot().semantic_fingerprint()
    }
}
