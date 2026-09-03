//! `sdk.ontology()` handle.

use std::collections::BTreeMap;

use os20_ontology_core::{
    ContentDigest, DerivedOntologyIndex, EffectiveProperty, ImpactReport, OntologyDiagnostic,
    OntologyDiff, OntologyFingerprints, OntologyPackage, OntologyRuntime, OntologySnapshot,
    Predicate, ReverseReferences, SemanticDomain, TypeId, TypeSummary, WhyCompatible, WhyType,
    ontology_diff,
};

/// Borrowed ontology API on [`crate::Os20Sdk`].
#[derive(Clone, Copy, Debug)]
pub struct OntologyHandle<'a> {
    runtime: &'a OntologyRuntime,
    named: &'a BTreeMap<String, OntologySnapshot>,
}

impl<'a> OntologyHandle<'a> {
    pub(crate) fn new(
        runtime: &'a OntologyRuntime,
        named: &'a BTreeMap<String, OntologySnapshot>,
    ) -> Self {
        Self { runtime, named }
    }

    pub(crate) fn named_snapshot(&self, name: &str) -> Option<&'a OntologySnapshot> {
        self.named.get(name)
    }

    pub(crate) fn named_snapshots(
        &self,
    ) -> impl Iterator<Item = (&'a String, &'a OntologySnapshot)> {
        self.named.iter()
    }

    /// Immutable snapshot for this SDK instance.
    pub fn snapshot(&self) -> &OntologySnapshot {
        self.runtime.snapshot()
    }

    /// Ontology packages in this snapshot.
    pub fn packages(&self) -> Vec<&OntologyPackage> {
        self.runtime.snapshot().packages().collect()
    }

    /// Lookup one ontology package by `@scope/name`.
    pub fn package(&self, name: &str) -> Option<&OntologyPackage> {
        let id = os20_ontology_core::PackageId::new(name).ok()?;
        self.runtime.snapshot().package(&id)
    }

    /// Resolve a type by id.
    pub fn resolve_type(&self, id: &TypeId) -> Option<TypeSummary> {
        self.runtime.snapshot().type_summary(id)
    }

    /// Search types (deterministic).
    pub fn find_types(&self, query: &str) -> Vec<TypeSummary> {
        self.runtime.snapshot().find_types(query)
    }

    /// Type summary.
    pub fn type_summary(&self, id: &TypeId) -> Option<TypeSummary> {
        self.runtime.snapshot().type_summary(id)
    }

    /// Transitive supertypes including self.
    pub fn supertypes(&self, id: &TypeId) -> Vec<TypeId> {
        self.runtime.snapshot().supertypes(id)
    }

    /// Transitive subtypes excluding self.
    pub fn subtypes(&self, id: &TypeId) -> Vec<TypeId> {
        self.runtime.snapshot().subtypes(id)
    }

    /// Effective properties.
    pub fn properties(&self, id: &TypeId) -> Vec<EffectiveProperty> {
        self.runtime.snapshot().properties(id)
    }

    /// Predicates.
    pub fn predicates(&self) -> Vec<&Predicate> {
        self.runtime.snapshot().predicates().collect()
    }

    /// Semantic domains.
    pub fn domains(&self) -> Vec<&SemanticDomain> {
        self.runtime.snapshot().domains().collect()
    }

    /// `ty` specializes `ancestor` (or equal).
    pub fn is_subtype(&self, ty: &TypeId, ancestor: &TypeId) -> bool {
        self.runtime.snapshot().is_subtype_of(ty, ancestor)
    }

    /// `value` assignable to `target`.
    pub fn is_assignable(&self, value: &TypeId, target: &TypeId) -> bool {
        self.runtime.snapshot().is_assignable_to(value, target)
    }

    /// Alias for [`Self::is_assignable`].
    pub fn assignable(&self, value: &TypeId, target: &TypeId) -> bool {
        self.is_assignable(value, target)
    }

    /// Same TypeId.
    pub fn is_same_type(&self, a: &TypeId, b: &TypeId) -> bool {
        self.runtime.snapshot().is_same_type(a, b)
    }

    /// Reverse references (type usages, predicates, package deps).
    pub fn references(&self) -> ReverseReferences {
        self.runtime.snapshot().reverse_references()
    }

    /// Diff this snapshot against `other` (typically an older lock).
    pub fn diff(&self, other: &OntologySnapshot) -> OntologyDiff {
        ontology_diff(other, self.runtime.snapshot())
    }

    /// Impact of a type change.
    pub fn impact(&self, id: &TypeId) -> ImpactReport {
        self.runtime.snapshot().impact_of_type(id)
    }

    /// Explain a type.
    pub fn why_type(&self, id: &TypeId) -> Option<WhyType> {
        self.runtime.snapshot().why_type(id)
    }

    /// Explain assignability.
    pub fn why_compatible(&self, from: &TypeId, to: &TypeId) -> WhyCompatible {
        self.runtime.snapshot().why_compatible(from, to)
    }

    /// Semantic fingerprint.
    pub fn fingerprint(&self) -> &ContentDigest {
        self.runtime.snapshot().semantic_fingerprint()
    }

    /// Source + semantic fingerprints.
    pub fn fingerprints(&self) -> &OntologyFingerprints {
        self.runtime.snapshot().fingerprints()
    }

    /// Structured diagnostics (missing ontology, cycles, unresolved).
    pub fn diagnostics(&self) -> &[OntologyDiagnostic] {
        self.runtime.snapshot().diagnostics()
    }

    /// Rebuildable derived index (SQLite analog).
    pub fn derived_index(&self) -> DerivedOntologyIndex {
        DerivedOntologyIndex::from_snapshot(self.runtime.snapshot())
    }
}
