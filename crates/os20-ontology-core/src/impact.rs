//! Reverse references and impact. Ontology augments impact traversal.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::fingerprint::ContentDigest;
use crate::identity::{PackageId, PropertyId, TypeId};
use crate::incremental::InvalidationReport;
use crate::package::TypeRef;
use crate::snapshot::OntologySnapshot;

/// Reverse indexes (derived; SQL would only accelerate these).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseReferences {
    /// Types used as feature types: T → (owner, property).
    pub type_usages: BTreeMap<String, Vec<(String, String)>>,
    /// Predicate ids.
    pub predicate_usages: BTreeMap<String, Vec<String>>,
    /// Ontology package import graph.
    pub package_dependencies: BTreeMap<String, Vec<String>>,
}

/// Impact of changing a type.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpactReport {
    /// Changed type.
    pub origin: TypeId,
    /// Properties typed as this type (and subtypes).
    pub typed_properties: Vec<(TypeId, PropertyId)>,
    /// Specialize descendants.
    pub descendants: Vec<TypeId>,
    /// Packages that import the origin package.
    pub downstream_packages: Vec<PackageId>,
}

impl OntologySnapshot {
    /// Build reverse indexes from IDs (not names).
    pub fn reverse_references(&self) -> ReverseReferences {
        let mut type_usages: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for t in self.type_ids() {
            if let Some(d) = self.type_decl(t) {
                for p in &d.properties {
                    if let TypeRef::Bound { id } = &p.ty {
                        type_usages
                            .entry(id.as_str().to_owned())
                            .or_default()
                            .push((t.as_str().to_owned(), p.id.as_str().to_owned()));
                    }
                }
            }
        }
        let mut predicate_usages: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for p in self.predicates() {
            predicate_usages
                .entry(p.relation.source_type.as_str().to_owned())
                .or_default()
                .push(p.id.as_str().to_owned());
            predicate_usages
                .entry(p.relation.target_type.as_str().to_owned())
                .or_default()
                .push(p.id.as_str().to_owned());
        }
        let mut package_dependencies: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for pkg in self.packages() {
            package_dependencies.insert(
                pkg.ontology_id.as_str().to_owned(),
                pkg.imports.iter().map(|i| i.as_str().to_owned()).collect(),
            );
        }
        ReverseReferences {
            type_usages,
            predicate_usages,
            package_dependencies,
        }
    }

    /// Impact traversal: type change → properties typed as it → descendants →
    /// importing packages.
    pub fn impact_of_type(&self, id: &TypeId) -> ImpactReport {
        let refs = self.reverse_references();
        let mut typed_properties = Vec::new();
        let mut walk = vec![id.clone()];
        walk.extend(self.subtypes(id));
        for t in &walk {
            if let Some(uses) = refs.type_usages.get(t.as_str()) {
                for (owner, prop) in uses {
                    if let (Ok(oid), Ok(pid)) = (TypeId::new(owner), PropertyId::new(prop)) {
                        typed_properties.push((oid, pid));
                    }
                }
            }
        }
        typed_properties.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        let origin_pkg = id.package().ok();
        let mut downstream_packages = Vec::new();
        if let Some(op) = &origin_pkg {
            for pkg in self.packages() {
                if pkg.imports.iter().any(|i| i == op) {
                    downstream_packages.push(pkg.ontology_id.clone());
                }
            }
        }
        ImpactReport {
            origin: id.clone(),
            typed_properties,
            descendants: self.subtypes(id),
            downstream_packages,
        }
    }
}

/// Cache keys mixing the ontology semantic fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedElementFingerprint {
    /// Ontology semantic fingerprint.
    pub ontology: ContentDigest,
    /// Element id.
    pub element: TypeId,
}

impl ResolvedElementFingerprint {
    /// Mix.
    pub fn new(ontology: ContentDigest, element: TypeId) -> Self {
        Self { ontology, element }
    }
}

/// Invalidate only dependent semantic caches when the ontology fingerprint moves.
pub fn invalidate_dependent_caches(
    before: &OntologySnapshot,
    after: &OntologySnapshot,
) -> InvalidationReport {
    if before.semantic_fingerprint() == after.semantic_fingerprint() {
        return InvalidationReport {
            semantic_noop: true,
            ..InvalidationReport::default()
        };
    }
    crate::incremental::invalidate_from_delta(
        before,
        after,
        &crate::source_graph::SourceGraphDelta {
            comment_only: false,
            ..Default::default()
        },
    )
}

/// Unrelated packages: types whose declaring package is outside `changed`.
pub fn unrelated_types_preserved(
    before: &OntologySnapshot,
    after: &OntologySnapshot,
    changed_packages: &BTreeSet<PackageId>,
) -> bool {
    for id in before.type_ids() {
        let Ok(pkg) = id.package() else { continue };
        if changed_packages.contains(&pkg) {
            continue;
        }
        let b = before.type_decl(id).map(|t| t.semantic_canonical());
        let a = after.type_decl(id).map(|t| t.semantic_canonical());
        if b != a {
            return false;
        }
    }
    true
}
