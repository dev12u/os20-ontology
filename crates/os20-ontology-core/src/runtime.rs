//! Ontology trait, runtime, and per-SDK-instance context.

use std::sync::Arc;

use crate::catalog::MemoryOntology;
use crate::fingerprint::ContentDigest;
use crate::identity::TypeId;
use crate::model::TypeSummary;
use crate::snapshot::OntologySnapshot;

/// Narrow ontology capability used by the resolver and SDK.
pub trait Ontology: Send + Sync {
    /// Lookup a type.
    fn lookup_type(&self, id: &TypeId) -> Option<TypeSummary>;
    /// Specialize-based subtyping.
    fn is_subtype_of(&self, ty: &TypeId, ancestor: &TypeId) -> bool;
    /// Assignability (`ty` usable where `target` is required).
    fn is_assignable_to(&self, ty: &TypeId, target: &TypeId) -> bool;
    /// Semantic ontology fingerprint.
    fn fingerprint(&self) -> &ContentDigest;
}

impl Ontology for OntologySnapshot {
    fn lookup_type(&self, id: &TypeId) -> Option<TypeSummary> {
        self.type_summary(id)
    }

    fn is_subtype_of(&self, ty: &TypeId, ancestor: &TypeId) -> bool {
        OntologySnapshot::is_subtype_of(self, ty, ancestor)
    }

    fn is_assignable_to(&self, ty: &TypeId, target: &TypeId) -> bool {
        OntologySnapshot::is_assignable_to(self, ty, target)
    }

    fn fingerprint(&self) -> &ContentDigest {
        self.semantic_fingerprint()
    }
}

impl Ontology for MemoryOntology {
    fn lookup_type(&self, id: &TypeId) -> Option<TypeSummary> {
        self.snapshot().lookup_type(id)
    }

    fn is_subtype_of(&self, ty: &TypeId, ancestor: &TypeId) -> bool {
        self.snapshot().is_subtype_of(ty, ancestor)
    }

    fn is_assignable_to(&self, ty: &TypeId, target: &TypeId) -> bool {
        self.snapshot().is_assignable_to(ty, target)
    }

    fn fingerprint(&self) -> &ContentDigest {
        self.snapshot().semantic_fingerprint()
    }
}

/// Per-instance ontology runtime. Never a process-global singleton.
#[derive(Clone, Debug)]
pub struct OntologyRuntime {
    snapshot: Arc<OntologySnapshot>,
}

impl OntologyRuntime {
    /// Wrap a snapshot.
    pub fn new(snapshot: OntologySnapshot) -> Self {
        Self {
            snapshot: Arc::new(snapshot),
        }
    }

    /// Bootstrap from [`MemoryOntology::core()`].
    pub fn bootstrap() -> Self {
        Self::new(MemoryOntology::core().into_snapshot())
    }

    /// Shared snapshot.
    pub fn snapshot(&self) -> &OntologySnapshot {
        &self.snapshot
    }

    /// In-memory runtime format (derived caches must match).
    pub fn format_version(&self) -> u32 {
        crate::versions::ONTOLOGY_RUNTIME_FORMAT_VERSION
    }

    /// Arc clone of the snapshot.
    pub fn snapshot_arc(&self) -> Arc<OntologySnapshot> {
        Arc::clone(&self.snapshot)
    }
}

impl Ontology for OntologyRuntime {
    fn lookup_type(&self, id: &TypeId) -> Option<TypeSummary> {
        self.snapshot.lookup_type(id)
    }

    fn is_subtype_of(&self, ty: &TypeId, ancestor: &TypeId) -> bool {
        self.snapshot.is_subtype_of(ty, ancestor)
    }

    fn is_assignable_to(&self, ty: &TypeId, target: &TypeId) -> bool {
        self.snapshot.is_assignable_to(ty, target)
    }

    fn fingerprint(&self) -> &ContentDigest {
        self.snapshot.semantic_fingerprint()
    }
}
