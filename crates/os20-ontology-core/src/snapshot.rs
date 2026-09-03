//! Immutable ontology snapshot. Identified by fingerprint. Not a process global.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::diagnostics::{DiagnosticCode, OntologyDiagnostic};
use crate::evolution::{EvolutionEdge, UnallocatedPattern};
use crate::fingerprint::{ContentDigest, OntologyFingerprints};
use crate::identity::{PackageId, PredicateId, PropertyId, SemanticDomainId, TypeId};
use crate::model::{
    EffectiveProperty, Predicate, QuantityType, SemanticDomain, TypeDecl, TypeSummary,
};
use crate::package::{
    OntologySourceKind, PackageReleaseRef, PackageRole, TypeRef, ValueOrigin, Visibility,
};
use crate::relations::{RelationEdge, RelationKind, RelationRegistry};

/// Public semantic representation of one ontology package (not a duplicate of
/// [`PackageReleaseRef`] identity).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyPackage {
    /// Semantic package id (`@os20/core`).
    pub ontology_id: PackageId,
    /// Ontology version supplied by this package.
    pub ontology_version: String,
    /// Source revision (commit).
    pub source_revision: Option<String>,
    /// Release coordinates from the existing lock/resolver.
    pub release: PackageReleaseRef,
    /// Imports (existing package machinery; names only here).
    pub imports: Vec<PackageId>,
    /// Package role (must be Ontology for OS20 ontology packages).
    pub role: PackageRole,
    /// Fingerprint of this package's semantic declarations.
    pub fingerprint: ContentDigest,
}

/// Immutable resolved ontology index.
#[derive(Clone, Debug)]
pub struct OntologySnapshot {
    source_kind: OntologySourceKind,
    packages: BTreeMap<PackageId, OntologyPackage>,
    types: BTreeMap<TypeId, TypeDecl>,
    domains: BTreeMap<SemanticDomainId, SemanticDomain>,
    predicates: BTreeMap<PredicateId, Predicate>,
    quantities: BTreeMap<TypeId, QuantityType>,
    specialize: BTreeMap<TypeId, BTreeSet<TypeId>>,
    specialize_incoming: BTreeMap<TypeId, BTreeSet<TypeId>>,
    mixins: BTreeMap<TypeId, BTreeSet<TypeId>>,
    evolution: Vec<EvolutionEdge>,
    unallocated: Vec<UnallocatedPattern>,
    fingerprints: OntologyFingerprints,
    diagnostics: Vec<OntologyDiagnostic>,
    relations: RelationRegistry,
}

impl OntologySnapshot {
    /// Fingerprints for this snapshot.
    pub fn fingerprints(&self) -> &OntologyFingerprints {
        &self.fingerprints
    }

    /// Semantic fingerprint (definitions only).
    pub fn semantic_fingerprint(&self) -> &ContentDigest {
        &self.fingerprints.semantic
    }

    /// Source fingerprint (package pins / revisions / comments).
    pub fn source_fingerprint(&self) -> &ContentDigest {
        &self.fingerprints.source
    }

    /// How this snapshot was loaded.
    pub fn source_kind(&self) -> &OntologySourceKind {
        &self.source_kind
    }

    /// Ontology packages, sorted by id.
    pub fn packages(&self) -> impl Iterator<Item = &OntologyPackage> {
        self.packages.values()
    }

    /// All type ids in this snapshot, sorted.
    pub fn type_ids(&self) -> impl Iterator<Item = &TypeId> {
        self.types.keys()
    }

    /// Package record by id.
    pub fn package(&self, id: &PackageId) -> Option<&OntologyPackage> {
        self.packages.get(id)
    }

    /// Diagnostics (missing ontology, cycles, unresolved types).
    pub fn diagnostics(&self) -> &[OntologyDiagnostic] {
        &self.diagnostics
    }

    /// Type lookup by id.
    pub fn type_decl(&self, id: &TypeId) -> Option<&TypeDecl> {
        self.types.get(id)
    }

    /// Summary for product surfaces.
    pub fn type_summary(&self, id: &TypeId) -> Option<TypeSummary> {
        let decl = self.types.get(id)?;
        let pkg = id.package().ok()?;
        let release = self.packages.get(&pkg);
        Some(TypeSummary {
            id: id.clone(),
            kind: decl.kind,
            package: pkg,
            ontology_version: release
                .map(|p| p.ontology_version.clone())
                .unwrap_or_else(|| "0.0.0".into()),
            source_revision: release.and_then(|p| p.source_revision.clone()),
            supertypes: sorted_types(decl.specializes.iter()),
            mixins: sorted_types(decl.mixins.iter()),
            domains: {
                let mut d = decl.domains.clone();
                d.sort();
                d
            },
            properties: {
                let mut p: Vec<_> = decl.properties.iter().map(|x| x.id.clone()).collect();
                p.sort();
                p
            },
            lifecycle: decl.lifecycle.clone(),
            provenance: decl.provenance.clone(),
        })
    }

    /// Find types whose local name or id contains `query` (deterministic order).
    pub fn find_types(&self, query: &str) -> Vec<TypeSummary> {
        let q = query.to_ascii_lowercase();
        self.types
            .keys()
            .filter(|id| {
                let vis_ok = self
                    .types
                    .get(*id)
                    .is_some_and(|t| t.visibility == Visibility::Public);
                vis_ok
                    && (id.as_str().to_ascii_lowercase().contains(&q)
                        || id.local_name().to_ascii_lowercase().contains(&q))
            })
            .filter_map(|id| self.type_summary(id))
            .collect()
    }

    /// Direct + transitive Specialize ancestors, `start` first, then sorted rest.
    pub fn supertypes(&self, start: &TypeId) -> Vec<TypeId> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        let mut q = VecDeque::new();
        q.push_back(start.clone());
        while let Some(cur) = q.pop_front() {
            if seen.len() >= crate::versions::MAX_SPECIALIZE_VISIT {
                break;
            }
            if !seen.insert(cur.clone()) {
                continue;
            }
            out.push(cur.clone());
            if let Some(parents) = self.specialize.get(&cur) {
                for p in parents {
                    q.push_back(p.clone());
                }
            }
        }
        out
    }

    /// Direct subtypes, sorted.
    pub fn direct_subtypes(&self, id: &TypeId) -> Vec<TypeId> {
        self.specialize_incoming
            .get(id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Transitive subtypes excluding self, sorted.
    pub fn subtypes(&self, id: &TypeId) -> Vec<TypeId> {
        let mut seen = BTreeSet::new();
        let mut q = VecDeque::new();
        q.push_back(id.clone());
        seen.insert(id.clone());
        while let Some(cur) = q.pop_front() {
            if seen.len() >= crate::versions::MAX_SPECIALIZE_VISIT {
                break;
            }
            if let Some(kids) = self.specialize_incoming.get(&cur) {
                for k in kids {
                    if seen.insert(k.clone()) {
                        q.push_back(k.clone());
                    }
                }
            }
        }
        seen.remove(id);
        seen.into_iter().collect()
    }

    /// Quantity type by id.
    pub fn quantity(&self, id: &TypeId) -> Option<&QuantityType> {
        self.quantities.get(id)
    }

    /// `ty` specializes `ancestor` (or is equal). Mixin is ignored.
    pub fn is_subtype_of(&self, ty: &TypeId, ancestor: &TypeId) -> bool {
        self.supertypes(ty).iter().any(|t| t == ancestor)
    }

    /// Assignability: `value` may be used where `target` is required.
    ///
    /// Mixins do not create assignability. Distinct quantity dimensions are
    /// incompatible even when both specialize `Thing`.
    pub fn is_assignable_to(&self, value: &TypeId, target: &TypeId) -> bool {
        matches!(
            self.assignability(value, target),
            crate::compatibility::Assignability::Same
                | crate::compatibility::Assignability::Subtype
        )
    }

    /// Mixin membership does not imply subtype.
    pub fn has_mixin(&self, ty: &TypeId, mixin: &TypeId) -> bool {
        self.mixins.get(ty).is_some_and(|s| s.contains(mixin))
            || self
                .supertypes(ty)
                .iter()
                .any(|anc| self.mixins.get(anc).is_some_and(|s| s.contains(mixin)))
    }

    /// Effective properties via Specialize + Declares (and mixin feature contribution).
    pub fn properties(&self, ty: &TypeId) -> Vec<EffectiveProperty> {
        let mut found: BTreeMap<PropertyId, EffectiveProperty> = BTreeMap::new();
        for anc in self.supertypes(ty) {
            if let Some(decl) = self.types.get(&anc) {
                for p in &decl.properties {
                    found.entry(p.id.clone()).or_insert(EffectiveProperty {
                        id: p.id.clone(),
                        declared_by: anc.clone(),
                        ty: p.ty.clone(),
                        default_origin: p.default_value.as_ref().map(|_| ValueOrigin::Default),
                    });
                }
                for mixin_id in &decl.mixins {
                    if let Some(md) = self.types.get(mixin_id) {
                        for p in &md.properties {
                            found.entry(p.id.clone()).or_insert(EffectiveProperty {
                                id: p.id.clone(),
                                declared_by: mixin_id.clone(),
                                ty: p.ty.clone(),
                                default_origin: p
                                    .default_value
                                    .as_ref()
                                    .map(|_| ValueOrigin::Default),
                            });
                        }
                    }
                }
            }
        }
        found.into_values().collect()
    }

    /// Predicates, sorted.
    pub fn predicates(&self) -> impl Iterator<Item = &Predicate> {
        self.predicates.values()
    }

    /// Semantic domains, sorted.
    pub fn domains(&self) -> impl Iterator<Item = &SemanticDomain> {
        self.domains.values()
    }

    /// Quantity types.
    pub fn quantities(&self) -> impl Iterator<Item = &QuantityType> {
        self.quantities.values()
    }

    /// Evolution edges.
    pub fn evolution(&self) -> &[EvolutionEdge] {
        &self.evolution
    }

    /// Unallocated patterns.
    pub fn unallocated(&self) -> &[UnallocatedPattern] {
        &self.unallocated
    }

    /// Core relation registry.
    pub fn relations(&self) -> &RelationRegistry {
        &self.relations
    }

    /// Specialize edges as relation occurrences.
    pub fn specialize_edges(&self) -> Vec<RelationEdge> {
        let mut edges = Vec::new();
        for (src, tgts) in &self.specialize {
            for t in tgts {
                edges.push(RelationEdge {
                    kind: RelationKind::Specialize,
                    source: src.clone(),
                    target: t.clone(),
                });
            }
        }
        edges
    }
}

/// Builder for snapshots (bootstrap or tests). Not a global.
#[derive(Clone, Debug, Default)]
pub struct OntologySnapshotBuilder {
    source_kind: Option<OntologySourceKind>,
    packages: BTreeMap<PackageId, OntologyPackage>,
    types: BTreeMap<TypeId, TypeDecl>,
    domains: BTreeMap<SemanticDomainId, SemanticDomain>,
    predicates: BTreeMap<PredicateId, Predicate>,
    quantities: BTreeMap<TypeId, QuantityType>,
    evolution: Vec<EvolutionEdge>,
    unallocated: Vec<UnallocatedPattern>,
    required_packages: Vec<PackageId>,
}

impl OntologySnapshotBuilder {
    /// New builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Bootstrap vs package-backed.
    pub fn source_kind(mut self, kind: OntologySourceKind) -> Self {
        self.source_kind = Some(kind);
        self
    }

    /// Require a package (missing → diagnostic).
    pub fn require_package(mut self, id: PackageId) -> Self {
        self.required_packages.push(id);
        self
    }

    /// Add a package record.
    pub fn package(mut self, pkg: OntologyPackage) -> Self {
        self.packages.insert(pkg.ontology_id.clone(), pkg);
        self
    }

    /// Add a type.
    pub fn type_decl(mut self, decl: TypeDecl) -> Self {
        self.types.insert(decl.id.clone(), decl);
        self
    }

    /// Add a domain.
    pub fn domain(mut self, d: SemanticDomain) -> Self {
        self.domains.insert(d.id.clone(), d);
        self
    }

    /// Add a predicate.
    pub fn predicate(mut self, p: Predicate) -> Self {
        self.predicates.insert(p.id.clone(), p);
        self
    }

    /// Add a quantity type.
    pub fn quantity(mut self, q: QuantityType) -> Self {
        self.quantities.insert(q.id.clone(), q);
        self
    }

    /// Evolution edge.
    pub fn evolution(mut self, e: EvolutionEdge) -> Self {
        self.evolution.push(e);
        self
    }

    /// Unallocated pattern.
    pub fn unallocated(mut self, p: UnallocatedPattern) -> Self {
        self.unallocated.push(p);
        self
    }

    /// Clone types with comments stripped (semantic-identical helper).
    pub fn with_comments_cleared(&self) -> Self {
        let mut c = self.clone();
        for t in c.types.values_mut() {
            t.comment = None;
        }
        c
    }

    /// Clone types adding a comment on `id`.
    pub fn with_comment_on(mut self, id: &TypeId, comment: &str) -> Self {
        if let Some(t) = self.types.get_mut(id) {
            t.comment = Some(comment.to_owned());
        }
        self
    }

    /// Finish.
    pub fn build(self) -> OntologySnapshot {
        let source_kind = self
            .source_kind
            .unwrap_or(OntologySourceKind::PackageBacked);
        let mut specialize: BTreeMap<TypeId, BTreeSet<TypeId>> = BTreeMap::new();
        let mut specialize_incoming: BTreeMap<TypeId, BTreeSet<TypeId>> = BTreeMap::new();
        let mut mixins: BTreeMap<TypeId, BTreeSet<TypeId>> = BTreeMap::new();
        let mut diagnostics = Vec::new();

        for pkg in &self.required_packages {
            if !self.packages.contains_key(pkg) {
                diagnostics.push(OntologyDiagnostic::missing_ontology(pkg.clone()));
            }
        }

        for (id, decl) in &self.types {
            for p in &decl.properties {
                if let TypeRef::Unresolved { written } = &p.ty {
                    diagnostics.push(OntologyDiagnostic::unresolved_type(written));
                }
            }
            let spec = specialize.entry(id.clone()).or_default();
            for s in &decl.specializes {
                spec.insert(s.clone());
                specialize_incoming
                    .entry(s.clone())
                    .or_default()
                    .insert(id.clone());
            }
            let mx = mixins.entry(id.clone()).or_default();
            for m in &decl.mixins {
                mx.insert(m.clone());
            }
        }

        diagnostics.extend(find_cycles(&specialize));
        if diagnostics.len() > crate::versions::MAX_ONTOLOGY_DIAGNOSTICS {
            diagnostics.truncate(crate::versions::MAX_ONTOLOGY_DIAGNOSTICS);
        }

        let mut packages = self.packages;
        for pkg in packages.values_mut() {
            let mut lines: Vec<String> = self
                .types
                .values()
                .filter(|t| t.id.package().ok().as_ref() == Some(&pkg.ontology_id))
                .map(TypeDecl::semantic_canonical)
                .collect();
            pkg.fingerprint = ContentDigest::hash_domain_sorted_lines(
                crate::versions::FINGERPRINT_PACKAGE,
                &mut lines,
            );
        }

        let fingerprints = compute_fingerprints(&packages, &self.types);

        OntologySnapshot {
            source_kind,
            packages,
            types: self.types,
            domains: self.domains,
            predicates: self.predicates,
            quantities: self.quantities,
            specialize,
            specialize_incoming,
            mixins,
            evolution: self.evolution,
            unallocated: self.unallocated,
            fingerprints,
            diagnostics,
            relations: RelationRegistry::core(),
        }
    }
}

fn sorted_types<'a>(iter: impl Iterator<Item = &'a TypeId>) -> Vec<TypeId> {
    let mut v: Vec<_> = iter.cloned().collect();
    v.sort();
    v
}

fn compute_fingerprints(
    packages: &BTreeMap<PackageId, OntologyPackage>,
    types: &BTreeMap<TypeId, TypeDecl>,
) -> OntologyFingerprints {
    let mut semantic: Vec<String> = types.values().map(TypeDecl::semantic_canonical).collect();
    let mut source: Vec<String> = Vec::new();
    for pkg in packages.values() {
        source.push(format!(
            "pkg\t{}\t{}\t{:?}\t{:?}\t{:?}",
            pkg.ontology_id,
            pkg.ontology_version,
            pkg.source_revision,
            pkg.release.commit,
            pkg.release.tree
        ));
        if let Some(d) = &pkg.release.manifest_digest {
            source.push(format!("manifest\t{}", d.as_str()));
        }
    }
    for t in types.values() {
        source.push(t.semantic_canonical());
        if let Some(c) = &t.comment {
            source.push(format!("comment\t{}\t{c}", t.id.as_str()));
        }
        source.push(format!(
            "file\t{}\t{}",
            t.id.as_str(),
            t.provenance.source_file
        ));
    }
    OntologyFingerprints {
        source: ContentDigest::hash_domain_sorted_lines(
            crate::versions::FINGERPRINT_SOURCE,
            &mut source,
        ),
        semantic: ContentDigest::hash_domain_sorted_lines(
            crate::versions::FINGERPRINT_SEMANTIC,
            &mut semantic,
        ),
    }
}

fn find_cycles(specialize: &BTreeMap<TypeId, BTreeSet<TypeId>>) -> Vec<OntologyDiagnostic> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color: BTreeMap<TypeId, Color> = BTreeMap::new();
    for (k, targets) in specialize {
        color.entry(k.clone()).or_insert(Color::White);
        for t in targets {
            color.entry(t.clone()).or_insert(Color::White);
        }
    }
    let mut out = Vec::new();
    let nodes: Vec<_> = color.keys().cloned().collect();
    for start in nodes {
        if color.get(&start) != Some(&Color::White) {
            continue;
        }
        let mut stack: Vec<(TypeId, usize)> = vec![(start.clone(), 0)];
        let mut path: Vec<TypeId> = vec![start.clone()];
        color.insert(start, Color::Gray);
        while let Some((node, child_idx)) = stack.last().cloned() {
            let children: Vec<TypeId> = specialize
                .get(&node)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default();
            if child_idx < children.len() {
                if let Some(frame) = stack.last_mut() {
                    frame.1 += 1;
                }
                let nxt = children[child_idx].clone();
                match color.get(&nxt).copied().unwrap_or(Color::White) {
                    Color::White => {
                        color.insert(nxt.clone(), Color::Gray);
                        path.push(nxt.clone());
                        stack.push((nxt, 0));
                    }
                    Color::Gray => {
                        let begin = path.iter().position(|x| x == &nxt).unwrap_or(0);
                        let mut cycle = path[begin..].to_vec();
                        cycle.push(nxt);
                        out.push(OntologyDiagnostic::cycle(cycle));
                    }
                    Color::Black => {}
                }
            } else {
                stack.pop();
                color.insert(node, Color::Black);
                path.pop();
            }
        }
    }
    let mut seen = BTreeSet::new();
    out.retain(|d| {
        if d.code != DiagnosticCode::SpecializationCycle {
            return true;
        }
        let key: Vec<_> = d.cycle.iter().map(|t| t.as_str().to_owned()).collect();
        seen.insert(key)
    });
    out
}
