//! Compile bound SourceGraph facts into [`OntologyPackageSnapshot`].
//!
//! Internals of name binding stay in this module. Product code uses
//! [`OntologyCompiler`].

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::catalog::MemoryOntology;
use crate::diagnostics::{DiagnosticCode, OntologyDiagnostic};
use crate::evolution::{EvolutionEdge, EvolutionKind};
use crate::fingerprint::ContentDigest;
use crate::identity::{
    ElementId, PackageId, PredicateId, PropertyId, SemanticDomainId, TypeId, UnitId,
};
use crate::lockfile::{LockedPackage, Os20Lockfile};
use crate::model::{
    OntologyTypeKind, Predicate, PropertyDecl, QuantityType, SemanticDomain, TypeDecl,
};
use crate::package::{
    LifecycleStatus, OntologySourceKind, PackageReleaseRef, PackageRole, Provenance, TypeRef,
};
use crate::relations::{Multiplicity, RelationAlgebra, RelationKind, RelationType};
use crate::snapshot::{OntologyPackage, OntologySnapshot, OntologySnapshotBuilder};
use crate::source_graph::{BoundElement, BoundSourceGraph, OntologyManifest};
use crate::source_tree::{
    FetchPolicy, OntologyRegistry, OntologyResolveMode, OntologySourceTree, SourceTreeError,
};

/// Compiled single-package snapshot.
#[derive(Clone, Debug)]
pub struct OntologyPackageSnapshot {
    /// Package record.
    pub package: OntologyPackage,
    /// Semantic types.
    pub types: BTreeMap<TypeId, TypeDecl>,
    /// Domains.
    pub domains: BTreeMap<SemanticDomainId, SemanticDomain>,
    /// Predicates.
    pub predicates: BTreeMap<PredicateId, Predicate>,
    /// Quantities.
    pub quantities: BTreeMap<TypeId, QuantityType>,
    /// Evolution.
    pub evolution: Vec<EvolutionEdge>,
    /// Diagnostics local to this package.
    pub diagnostics: Vec<OntologyDiagnostic>,
}

/// Compiler output for a resolved ontology set.
#[derive(Clone, Debug)]
pub struct OntologyCompileResult {
    /// Combined runtime snapshot.
    pub snapshot: OntologySnapshot,
    /// Per-package snapshots in deterministic order.
    pub packages: Vec<OntologyPackageSnapshot>,
    /// Diagnostics.
    pub diagnostics: Vec<OntologyDiagnostic>,
}

/// Compile request (existing resolver/lock; no second lockfile).
#[derive(Clone)]
pub struct OntologyCompileRequest {
    /// Resolve mode.
    pub mode: OntologyResolveMode,
    /// Git fetch policy.
    pub fetch: FetchPolicy,
    /// Lock pins.
    pub lock: Option<Os20Lockfile>,
    /// Workspace / known manifests.
    pub manifests: Vec<OntologyManifest>,
    /// Bound graphs keyed by package id string (language frontend output).
    pub graphs: BTreeMap<String, BoundSourceGraph>,
    /// Required ontology packages.
    pub required: Vec<PackageId>,
    /// Source tree for Git-backed loads.
    pub tree: Option<Arc<dyn OntologySourceTree>>,
    /// Existing registry (Update only).
    pub registry: Option<Arc<dyn OntologyRegistry>>,
    /// Commit used for workspace graphs.
    pub workspace_commit: Option<String>,
}

impl std::fmt::Debug for OntologyCompileRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OntologyCompileRequest")
            .field("mode", &self.mode)
            .field("fetch", &self.fetch)
            .finish_non_exhaustive()
    }
}

impl Default for OntologyCompileRequest {
    fn default() -> Self {
        Self {
            mode: OntologyResolveMode::LockedOffline,
            fetch: FetchPolicy::Never,
            lock: None,
            manifests: Vec::new(),
            graphs: BTreeMap::new(),
            required: Vec::new(),
            tree: None,
            registry: None,
            workspace_commit: None,
        }
    }
}

/// Ontology compiler. Does not parse KerML/SysML.
pub struct OntologyCompiler;

impl OntologyCompiler {
    /// Compile a resolved set into a package-backed [`OntologySnapshot`].
    pub fn compile(request: OntologyCompileRequest) -> OntologyCompileResult {
        let mut diagnostics = Vec::new();
        let mut selected = select_packages(&request, &mut diagnostics);

        if let Err(cycle) = topo_sort_packages(&mut selected) {
            diagnostics.push(
                OntologyDiagnostic::new(
                    DiagnosticCode::OntologyDependencyCycle,
                    format!("ontology package dependency cycle involving `{cycle}`"),
                )
                .with_package(cycle),
            );
        }

        let mut package_snaps = Vec::new();
        let mut known: BTreeMap<TypeId, PackageId> = BTreeMap::new();
        let bootstrap = MemoryOntology::core();
        let protected: BTreeSet<TypeId> = bootstrap.snapshot().type_ids().cloned().collect();

        // Binding environment grows in bootstrap order: stdlib + core, then deps.
        let mut bind_env: BTreeMap<String, TypeId> = BTreeMap::new();
        for id in bootstrap.snapshot().type_ids() {
            bind_env.insert(id.as_str().to_owned(), id.clone());
            bind_env.insert(id.local_name().to_owned(), id.clone());
        }

        for spec in &selected {
            let graph = load_graph(&request, spec, &mut diagnostics);
            let Some(graph) = graph else {
                continue;
            };
            let snap = compile_one(
                spec,
                &graph,
                &bind_env,
                &protected,
                &mut known,
                &mut diagnostics,
            );
            for (id, decl) in &snap.types {
                bind_env.insert(id.as_str().to_owned(), id.clone());
                bind_env.insert(id.local_name().to_owned(), id.clone());
                let _ = decl;
            }
            package_snaps.push(snap);
        }

        let combined = combine_snapshots(&bootstrap, &package_snaps, &selected, &diagnostics);
        let mut all_diags = diagnostics;
        all_diags.extend(combined.diagnostics().iter().cloned());
        all_diags.extend(package_snaps.iter().flat_map(|p| p.diagnostics.clone()));
        OntologyCompileResult {
            snapshot: combined,
            packages: package_snaps,
            diagnostics: all_diags,
        }
    }
}

#[derive(Clone, Debug)]
struct PackageSpec {
    manifest: OntologyManifest,
    release: PackageReleaseRef,
}

fn select_packages(
    request: &OntologyCompileRequest,
    diagnostics: &mut Vec<OntologyDiagnostic>,
) -> Vec<PackageSpec> {
    let mut specs = Vec::new();
    for m in &request.manifests {
        if m.role != PackageRole::Ontology && m.role != PackageRole::StandardLibrary {
            diagnostics.push(
                OntologyDiagnostic::new(
                    DiagnosticCode::IncompatibleOntologyRole,
                    format!(
                        "package `{}` is not an ontology package (role {:?})",
                        m.name, m.role
                    ),
                )
                .with_package(m.name.clone()),
            );
            continue;
        }
        let release = release_for(request, m, diagnostics);
        specs.push(PackageSpec {
            manifest: m.clone(),
            release,
        });
    }
    for req in &request.required {
        if !specs.iter().any(|s| &s.manifest.name == req) {
            if request.mode == OntologyResolveMode::LockedOffline
                || request.mode == OntologyResolveMode::Locked
            {
                diagnostics.push(OntologyDiagnostic::missing_ontology(req.clone()));
            } else if let (Some(reg), Some(lock)) = (&request.registry, &request.lock) {
                let _ = lock;
                match reg.resolve_version(req.as_str(), "*") {
                    Ok(_) => {}
                    Err(_) => diagnostics.push(OntologyDiagnostic::missing_ontology(req.clone())),
                }
            } else {
                diagnostics.push(OntologyDiagnostic::missing_ontology(req.clone()));
            }
        }
    }
    specs.sort_by(|a, b| a.manifest.name.as_str().cmp(b.manifest.name.as_str()));
    specs
}

fn release_for(
    request: &OntologyCompileRequest,
    manifest: &OntologyManifest,
    diagnostics: &mut Vec<OntologyDiagnostic>,
) -> PackageReleaseRef {
    if let Some(lock) = &request.lock {
        if let Some(pin) = lock.get(manifest.name.as_str()) {
            if request.mode == OntologyResolveMode::Update {
                if let Some(reg) = &request.registry {
                    let _ = reg.resolve_version(manifest.name.as_str(), &manifest.version);
                }
            }
            return pin.release_ref().unwrap_or_else(|_| {
                PackageReleaseRef::bootstrap(manifest.name.clone(), &manifest.version)
            });
        }
        if request.mode != OntologyResolveMode::Update {
            diagnostics.push(OntologyDiagnostic::missing_ontology(manifest.name.clone()));
        }
    }
    if request.mode == OntologyResolveMode::Update {
        if let Some(reg) = &request.registry {
            if let Ok(v) = reg.resolve_version(manifest.name.as_str(), &manifest.version) {
                let mut r = PackageReleaseRef::bootstrap(manifest.name.clone(), &v);
                r.version = v;
                return r;
            }
        }
    }
    PackageReleaseRef::bootstrap(manifest.name.clone(), &manifest.version)
}

fn topo_sort_packages(specs: &mut Vec<PackageSpec>) -> Result<(), PackageId> {
    let mut by_id: BTreeMap<String, PackageSpec> = BTreeMap::new();
    for s in specs.drain(..) {
        by_id.insert(s.manifest.name.as_str().to_owned(), s);
    }
    let mut indeg: BTreeMap<String, usize> = BTreeMap::new();
    let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, spec) in &by_id {
        indeg.entry(id.clone()).or_insert(0);
        for dep in spec
            .manifest
            .dependencies
            .iter()
            .chain(spec.manifest.optional_dependencies.iter())
        {
            let d = dep.as_str().to_owned();
            if by_id.contains_key(&d) {
                edges.entry(d.clone()).or_default().push(id.clone());
                *indeg.entry(id.clone()).or_insert(0) += 1;
                indeg.entry(d).or_insert(0);
            }
        }
    }
    let mut ready: Vec<String> = indeg
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(k, _)| k.clone())
        .collect();
    ready.sort();
    let mut out = Vec::new();
    while let Some(n) = {
        if ready.is_empty() {
            None
        } else {
            Some(ready.remove(0))
        }
    } {
        out.push(n.clone());
        if let Some(children) = edges.get(&n) {
            let mut ch = children.clone();
            ch.sort();
            for c in ch {
                if let Some(d) = indeg.get_mut(&c) {
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        ready.push(c);
                        ready.sort();
                    }
                }
            }
        }
    }
    if out.len() != by_id.len() {
        let leftover = by_id
            .keys()
            .find(|k| !out.contains(k))
            .cloned()
            .unwrap_or_else(|| "@os20/unknown".into());
        *specs = by_id.into_values().collect();
        specs.sort_by(|a, b| a.manifest.name.as_str().cmp(b.manifest.name.as_str()));
        return Err(PackageId::new(&leftover).unwrap_or_else(|_| PackageId::os20_core()));
    }
    *specs = out.into_iter().filter_map(|id| by_id.remove(&id)).collect();
    Ok(())
}

fn load_graph(
    request: &OntologyCompileRequest,
    spec: &PackageSpec,
    diagnostics: &mut Vec<OntologyDiagnostic>,
) -> Option<BoundSourceGraph> {
    if let Some(g) = request.graphs.get(spec.manifest.name.as_str()) {
        return Some(g.clone());
    }
    let tree = request.tree.as_ref()?;
    let fetch = request.fetch;
    let commit = spec
        .release
        .commit
        .as_deref()
        .or(request.workspace_commit.as_deref());
    let root = &spec.manifest.package_root;
    let files = match tree.list_files(commit, root, fetch) {
        Ok(f) => f,
        Err(SourceTreeError::OfflineMissing(p)) => {
            diagnostics.push(
                OntologyDiagnostic::new(
                    DiagnosticCode::MissingSourceOffline,
                    format!("ontology source `{p}` is not in the Git cache"),
                )
                .with_package(spec.manifest.name.clone()),
            );
            return None;
        }
        Err(e) => {
            diagnostics.push(
                OntologyDiagnostic::new(DiagnosticCode::MalformedOntologySource, e.to_string())
                    .with_package(spec.manifest.name.clone()),
            );
            return None;
        }
    };
    let graph_path = files
        .iter()
        .find(|p| p.ends_with("ontology.sourcegraph.json"));
    let Some(path) = graph_path else {
        diagnostics.push(
            OntologyDiagnostic::new(
                DiagnosticCode::MalformedOntologySource,
                "no bound SourceGraph (ontology.sourcegraph.json); language frontend required",
            )
            .with_package(spec.manifest.name.clone()),
        );
        return None;
    };
    match tree.read_blob(commit, path, fetch) {
        Ok(Some(bytes)) => match serde_json::from_slice(&bytes) {
            Ok(g) => Some(g),
            Err(e) => {
                diagnostics.push(
                    OntologyDiagnostic::new(
                        DiagnosticCode::MalformedOntologySource,
                        format!("malformed SourceGraph: {e}"),
                    )
                    .with_package(spec.manifest.name.clone()),
                );
                None
            }
        },
        Ok(None) | Err(SourceTreeError::OfflineMissing(_)) => {
            diagnostics.push(
                OntologyDiagnostic::new(
                    DiagnosticCode::MissingSourceOffline,
                    format!("missing Git blob `{path}`"),
                )
                .with_package(spec.manifest.name.clone()),
            );
            None
        }
        Err(e) => {
            diagnostics.push(
                OntologyDiagnostic::new(DiagnosticCode::MalformedOntologySource, e.to_string())
                    .with_package(spec.manifest.name.clone()),
            );
            None
        }
    }
}

fn compile_one(
    spec: &PackageSpec,
    graph: &BoundSourceGraph,
    bind_env: &BTreeMap<String, TypeId>,
    protected: &BTreeSet<TypeId>,
    known: &mut BTreeMap<TypeId, PackageId>,
    global_diags: &mut Vec<OntologyDiagnostic>,
) -> OntologyPackageSnapshot {
    let mut diagnostics = Vec::new();
    let pkg = spec.manifest.name.clone();
    let mut types = BTreeMap::new();
    let mut domains = BTreeMap::new();
    let mut predicates = BTreeMap::new();
    let mut quantities = BTreeMap::new();
    let mut evolution = Vec::new();

    let mut local_names: BTreeMap<String, TypeId> = BTreeMap::new();
    for el in &graph.elements {
        match qualified_id(&pkg, el) {
            Ok(id) => {
                local_names.insert(el.name.clone(), id.clone());
                local_names.insert(id.as_str().to_owned(), id);
            }
            Err(_) => diagnostics.push(OntologyDiagnostic::unresolved_type(&el.name)),
        }
    }

    let resolve = |written: &str| -> Option<TypeId> {
        if let Ok(id) = TypeId::new(written) {
            return Some(id);
        }
        if let Some(id) = local_names.get(written) {
            return Some(id.clone());
        }
        bind_env.get(written).cloned()
    };

    for el in &graph.elements {
        let Ok(id) = qualified_id(&pkg, el) else {
            continue;
        };
        if let Some(owner) = known.get(&id) {
            if owner != &pkg {
                diagnostics.push(
                    OntologyDiagnostic::new(
                        DiagnosticCode::DuplicateOntologyId,
                        format!(
                            "duplicate ontology id `{}` (already defined by `{owner}`)",
                            id.as_str()
                        ),
                    )
                    .with_element(id.as_element().clone())
                    .with_package(pkg.clone()),
                );
                continue;
            }
        }
        let owner_ok = id.package().ok().as_ref() == Some(&pkg);
        if protected.contains(&id) && !owner_ok {
            diagnostics.push(
                OntologyDiagnostic::new(
                    DiagnosticCode::ProtectedRedefinition,
                    format!(
                        "illegal redefinition of protected bootstrap id `{}`",
                        id.as_str()
                    ),
                )
                .with_element(id.as_element().clone())
                .with_package(pkg.clone()),
            );
            continue;
        }
        known.insert(id.clone(), pkg.clone());

        if el.kind.eq_ignore_ascii_case("Domain") {
            if el.name.is_empty() {
                diagnostics.push(
                    OntologyDiagnostic::new(
                        DiagnosticCode::InvalidDomainDeclaration,
                        "semantic domain name is empty",
                    )
                    .with_package(pkg.clone()),
                );
                continue;
            }
            let did = SemanticDomainId::from_element(id.as_element().clone());
            domains.insert(
                did.clone(),
                SemanticDomain {
                    id: did,
                    name: el.name.clone(),
                    defined_in: pkg.clone(),
                },
            );
            continue;
        }

        let kind = map_kind(&el.kind);
        let mut specializes = Vec::new();
        for s in &el.specializes {
            match resolve(s) {
                Some(t) => specializes.push(t),
                None => diagnostics.push(OntologyDiagnostic::unresolved_type(s)),
            }
        }
        for rel in graph.relations.iter().filter(|r| r.source == el.name) {
            if rel.kind == RelationKind::Specialize.as_str() {
                if let Some(t) = resolve(&rel.target) {
                    if !specializes.contains(&t) {
                        specializes.push(t);
                    }
                }
            }
        }
        let mut mixins = Vec::new();
        for m in &el.mixins {
            match resolve(m) {
                Some(t) => mixins.push(t),
                None => diagnostics.push(OntologyDiagnostic::unresolved_type(m)),
            }
        }
        let mut domain_ids = Vec::new();
        for d in &el.domains {
            if d.is_empty() {
                diagnostics.push(
                    OntologyDiagnostic::new(
                        DiagnosticCode::InvalidDomainDeclaration,
                        format!("invalid domain on `{}`", el.name),
                    )
                    .with_package(pkg.clone()),
                );
                continue;
            }
            if let Ok(id) = SemanticDomainId::new(d) {
                domain_ids.push(id);
            } else {
                domain_ids.push(SemanticDomainId::catalog(&pkg, d));
            }
        }

        let mut properties = Vec::new();
        for f in &el.features {
            let ty = match resolve(&f.ty) {
                Some(t) => TypeRef::Bound { id: t },
                None => {
                    diagnostics.push(OntologyDiagnostic::unresolved_type(&f.ty));
                    TypeRef::Unresolved {
                        written: f.ty.clone(),
                    }
                }
            };
            properties.push(PropertyDecl {
                id: PropertyId::catalog(&pkg, &f.name),
                ty,
                multiplicity: Some(Multiplicity {
                    lower: 0,
                    upper: Some(1),
                }),
                default_value: f.default_value.clone(),
                visibility: el.visibility,
            });
        }

        let mut lifecycle = el.lifecycle.clone();
        if let Some(rep) = &el.replaced_by {
            match resolve(rep) {
                Some(t) => {
                    lifecycle = LifecycleStatus::Deprecated {
                        replacement: Some(t.as_element().clone()),
                    };
                    evolution.push(EvolutionEdge {
                        kind: EvolutionKind::ReplacedBy,
                        from: id.as_element().clone(),
                        to: t.as_element().clone(),
                    });
                }
                None => diagnostics.push(
                    OntologyDiagnostic::new(
                        DiagnosticCode::EvolutionTargetMissing,
                        format!("evolution mapping target `{rep}` is missing"),
                    )
                    .with_element(id.as_element().clone()),
                ),
            }
        }
        for t in &el.split_into {
            match resolve(t) {
                Some(to) => evolution.push(EvolutionEdge {
                    kind: EvolutionKind::SplitInto,
                    from: id.as_element().clone(),
                    to: to.as_element().clone(),
                }),
                None => diagnostics.push(
                    OntologyDiagnostic::new(
                        DiagnosticCode::EvolutionTargetMissing,
                        format!("split target `{t}` is missing"),
                    )
                    .with_element(id.as_element().clone()),
                ),
            }
        }

        let provenance = Provenance {
            package: pkg.clone(),
            source_file: el.file.clone(),
            span: el.span.clone(),
            commit: spec.release.commit.clone(),
            tree: spec.release.tree.clone(),
        };

        if kind == OntologyTypeKind::Predicate {
            let pred_id = PredicateId::from_element(id.as_element().clone());
            let src = specializes.first().cloned().unwrap_or_else(TypeId::thing);
            let tgt = el
                .features
                .first()
                .and_then(|f| resolve(&f.ty))
                .unwrap_or_else(TypeId::thing);
            predicates.insert(
                pred_id.clone(),
                Predicate {
                    id: pred_id.clone(),
                    relation: RelationType {
                        id: pred_id,
                        source_type: src,
                        target_type: tgt,
                        cardinality: None,
                        inverse: el.inverse.as_ref().and_then(|inv| {
                            if inv.contains('#') {
                                PredicateId::new(inv).ok()
                            } else {
                                Some(PredicateId::catalog(&pkg, inv))
                            }
                        }),
                        algebra: RelationAlgebra {
                            symmetric: el.symmetric,
                            transitive: el.transitive,
                        },
                        domains: domain_ids.clone(),
                    },
                },
            );
        }

        if kind == OntologyTypeKind::Quantity {
            if let Some(dim) = &el.quantity_dimension {
                let vt = el
                    .value_type
                    .as_deref()
                    .and_then(resolve)
                    .unwrap_or_else(|| TypeId::new("@omg/kerml#Real").expect("real"));
                quantities.insert(
                    id.clone(),
                    QuantityType {
                        id: id.clone(),
                        dimension: dim.clone(),
                        preferred_unit: el
                            .preferred_unit
                            .as_deref()
                            .map(|u| UnitId::catalog(&pkg, u)),
                        value_type: vt,
                    },
                );
            }
        }

        types.insert(
            id.clone(),
            TypeDecl {
                id,
                kind,
                specializes,
                mixins,
                domains: domain_ids,
                properties,
                lifecycle,
                visibility: el.visibility,
                provenance,
                comment: el.comment.clone(),
            },
        );
    }

    global_diags.extend(diagnostics.iter().cloned());
    let mut fp_lines: Vec<String> = types.values().map(TypeDecl::semantic_canonical).collect();
    let fingerprint = ContentDigest::hash_domain_sorted_lines(
        crate::versions::FINGERPRINT_PACKAGE,
        &mut fp_lines,
    );
    OntologyPackageSnapshot {
        package: OntologyPackage {
            ontology_id: pkg,
            ontology_version: spec.manifest.version.clone(),
            source_revision: spec.release.commit.clone(),
            release: spec.release.clone(),
            imports: spec.manifest.dependencies.clone(),
            role: spec.manifest.role,
            fingerprint,
        },
        types,
        domains,
        predicates,
        quantities,
        evolution,
        diagnostics,
    }
}

fn qualified_id(
    pkg: &PackageId,
    el: &BoundElement,
) -> Result<TypeId, crate::identity::IdentityError> {
    if let Some(q) = &el.qualified {
        TypeId::new(q)
    } else {
        Ok(TypeId::from_element(ElementId::qualified(pkg, &el.name)?))
    }
}

fn map_kind(kind: &str) -> OntologyTypeKind {
    match kind.to_ascii_lowercase().as_str() {
        "datatype" | "data_type" => OntologyTypeKind::DataType,
        "structure" => OntologyTypeKind::Structure,
        "valuetype" | "value_type" => OntologyTypeKind::ValueType,
        "enumeration" => OntologyTypeKind::Enumeration,
        "quantity" => OntologyTypeKind::Quantity,
        "unit" => OntologyTypeKind::Unit,
        "predicate" => OntologyTypeKind::Predicate,
        "relationtype" | "relation_type" => OntologyTypeKind::RelationType,
        _ => OntologyTypeKind::Classifier,
    }
}

fn combine_snapshots(
    bootstrap: &MemoryOntology,
    compiled: &[OntologyPackageSnapshot],
    selected: &[PackageSpec],
    extra_diags: &[OntologyDiagnostic],
) -> OntologySnapshot {
    let mut b = OntologySnapshotBuilder::new().source_kind(if compiled.is_empty() {
        OntologySourceKind::Bootstrap
    } else {
        OntologySourceKind::PackageBacked
    });
    let compiled_ids: BTreeSet<TypeId> = compiled
        .iter()
        .flat_map(|p| p.types.keys().cloned())
        .collect();
    let compiled_packages: BTreeSet<PackageId> = compiled
        .iter()
        .map(|p| p.package.ontology_id.clone())
        .collect();
    for id in bootstrap.snapshot().type_ids() {
        if compiled_ids.contains(id) {
            continue;
        }
        if let Ok(pkg) = id.package() {
            if compiled_packages.contains(&pkg) {
                continue;
            }
        }
        if let Some(sum) = bootstrap.snapshot().type_decl(id) {
            b = b.type_decl(sum.clone());
        }
    }
    for spec in selected {
        b = b.require_package(spec.manifest.name.clone());
        b = b.package(OntologyPackage {
            ontology_id: spec.manifest.name.clone(),
            ontology_version: spec.manifest.version.clone(),
            source_revision: spec.release.commit.clone(),
            release: spec.release.clone(),
            imports: spec.manifest.dependencies.clone(),
            role: spec.manifest.role,
            fingerprint: ContentDigest::hash_bytes(b""),
        });
    }
    for p in compiled {
        b = b.package(p.package.clone());
        for t in p.types.values() {
            b = b.type_decl(t.clone());
        }
        for d in p.domains.values() {
            b = b.domain(d.clone());
        }
        for pred in p.predicates.values() {
            b = b.predicate(pred.clone());
        }
        for q in p.quantities.values() {
            b = b.quantity(q.clone());
        }
        for e in &p.evolution {
            b = b.evolution(e.clone());
        }
    }
    let snap = b.build();
    let _ = extra_diags;
    snap
}

/// Parse a tiny `os20.toml` subset for ontology role detection (not a second syntax).
pub fn parse_ontology_manifest(toml: &str, package_root: &str) -> Option<OntologyManifest> {
    let mut name = None;
    let mut version = String::from("0.0.0");
    let mut role = PackageRole::Dependency;
    let mut dependencies = Vec::new();
    let mut optional_dependencies = Vec::new();
    let mut in_optional = false;
    for raw in toml.lines() {
        let line = raw.trim();
        if line.starts_with('[') {
            in_optional = line.contains("optional");
            continue;
        }
        if let Some(v) = line.strip_prefix("name") {
            if let Some(s) = v.split('=').nth(1) {
                name = PackageId::new(s.trim().trim_matches('"')).ok();
            }
        }
        if let Some(v) = line.strip_prefix("version") {
            if let Some(s) = v.split('=').nth(1) {
                version = s.trim().trim_matches('"').to_owned();
            }
        }
        if line.starts_with("role") || line.starts_with("kind") {
            if line.contains("ontology") {
                role = PackageRole::Ontology;
            }
            if line.contains("standard") {
                role = PackageRole::StandardLibrary;
            }
        }
        if line.contains("@")
            && (line.contains("dependencies") || in_optional || line.contains('='))
        {
            if let Some(at) = line.find('@') {
                let rest = &line[at..];
                let token = rest.split(['"', ' ', ',', '}']).next().unwrap_or("");
                if let Some((pkg, _)) = token.split_once('@').filter(|(a, _)| a.is_empty()) {
                    let _ = pkg;
                }
                let pkg_part = if let Some(end) = rest.find(['"', ' ', '\t']) {
                    &rest[..end]
                } else {
                    rest
                };
                let pkg_only = pkg_part.split('@').next().unwrap_or(pkg_part);
                if pkg_only.starts_with('@') {
                    if let Ok(id) = PackageId::new(pkg_only) {
                        if in_optional {
                            optional_dependencies.push(id);
                        } else if !line.starts_with("name") {
                            dependencies.push(id);
                        }
                    }
                }
            }
        }
    }
    let name = name?;
    dependencies.retain(|d| d != &name);
    Some(OntologyManifest {
        name,
        version,
        role,
        dependencies,
        optional_dependencies,
        package_root: package_root.to_owned(),
    })
}

/// Load lockfile from JSON bytes (`os20.lock` may be TOML in products; tests use JSON).
pub fn lockfile_from_locked_packages(packages: Vec<LockedPackage>) -> Os20Lockfile {
    Os20Lockfile {
        format: 1,
        package: packages,
    }
}
