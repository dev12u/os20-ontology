//! Ontology track Prompt 2 — package-backed compilation.

use std::collections::BTreeMap;
use std::sync::Arc;

use os20_ontology_core::{
    BoundElement, BoundSourceGraph, DiagnosticCode, FetchPolicy, GitObjectCache, LockedPackage,
    OntologyCompileRequest, OntologyCompiler, OntologyManifest, OntologyResolveMode,
    OntologySourceKind, PackageId, PackageRole, RecordingRegistry, SourceGraphDelta, TypeId,
    Visibility, authored_source, behaviour_graph, behaviour_manifest, comment_only_edit,
    core_graph, core_manifest, electrical_graph, electrical_manifest, invalidate_from_delta,
    kerml_graph, kerml_manifest, mechanical_graph, mechanical_manifest, parse_ontology_manifest,
    physics_graph, physics_manifest, physics_v2_graph, physics_v2_manifest, reindex,
};
use os20_sdk::Os20Sdk;

fn tid(raw: &str) -> TypeId {
    TypeId::new(raw).unwrap()
}

fn standard_manifests() -> Vec<OntologyManifest> {
    vec![
        kerml_manifest(),
        core_manifest(),
        physics_manifest(),
        electrical_manifest(),
        mechanical_manifest(),
        behaviour_manifest(),
    ]
}

fn standard_graphs() -> BTreeMap<String, BoundSourceGraph> {
    BTreeMap::from([
        ("@omg/kerml".into(), kerml_graph()),
        ("@os20/core".into(), core_graph()),
        ("@os20/physics".into(), physics_graph()),
        ("@os20/electrical".into(), electrical_graph()),
        ("@os20/mechanical".into(), mechanical_graph()),
        ("@os20/behaviour".into(), behaviour_graph()),
    ])
}

fn compile_standard() -> os20_ontology_core::OntologyCompileResult {
    OntologyCompiler::compile(OntologyCompileRequest {
        mode: OntologyResolveMode::Locked,
        fetch: FetchPolicy::Never,
        manifests: standard_manifests(),
        graphs: standard_graphs(),
        ..OntologyCompileRequest::default()
    })
}

fn seed_git_cache(cache: &GitObjectCache, commit: &str) {
    for (pkg, graph) in standard_graphs() {
        let root = match pkg.as_str() {
            "@omg/kerml" => "kerml",
            "@os20/core" => "core",
            "@os20/physics" => "physics",
            "@os20/electrical" => "electrical",
            "@os20/mechanical" => "mechanical",
            "@os20/behaviour" => "behaviour",
            _ => "pkg",
        };
        let json = serde_json::to_vec(&graph).unwrap();
        cache.insert(
            commit,
            &format!("{root}/src/ontology.sourcegraph.json"),
            json,
        );
        cache.insert(
            commit,
            &format!("{root}/src/Authored.sysml"),
            authored_source(&pkg).as_bytes().to_vec(),
        );
        cache.insert(
            commit,
            &format!("{root}/os20.toml"),
            format!(
                "[package]\nname = \"{pkg}\"\nversion = \"0.1.0\"\nedition = \"2026\"\nrole = \"ontology\"\n"
            )
            .into_bytes(),
        );
    }
}

#[test]
fn compiles_classifiers_features_specialize_quantities_domains() {
    let r = compile_standard();
    let sdk = Os20Sdk::from_compile(r);
    assert_eq!(
        sdk.ontology().snapshot().source_kind(),
        &OntologySourceKind::PackageBacked
    );
    assert!(
        sdk.ontology()
            .resolve_type(&tid("@os20/core#Thing"))
            .is_some()
    );
    assert!(
        sdk.ontology()
            .is_subtype(&tid("@os20/core#Machine"), &tid("@os20/core#PhysicalThing"))
    );
    assert!(sdk.ontology().is_subtype(
        &tid("@os20/electrical#ElectricMachine"),
        &tid("@os20/core#Machine")
    ));
    assert!(sdk.ontology().is_subtype(
        &tid("@os20/electrical#ElectricMachine"),
        &tid("@os20/core#Thing")
    ));
    assert!(
        sdk.ontology()
            .resolve_type(&tid("@os20/physics#Mass"))
            .unwrap()
            .kind
            == os20_ontology_core::OntologyTypeKind::Quantity
    );
    assert!(sdk.ontology().package("@os20/electrical").is_some());
    assert!(!sdk.ontology().domains().is_empty());
}

#[test]
fn package_role_ontology_from_manifest() {
    let toml = "[package]\nname = \"@os20/core\"\nversion = \"0.1.0\"\nrole = \"ontology\"\n";
    let m = parse_ontology_manifest(toml, "core").unwrap();
    assert_eq!(m.role, PackageRole::Ontology);
}

#[test]
fn incompatible_role_diagnostic() {
    let mut m = core_manifest();
    m.role = PackageRole::Dependency;
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![m],
        graphs: BTreeMap::from([("@os20/core".into(), core_graph())]),
        ..OntologyCompileRequest::default()
    });
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::IncompatibleOntologyRole && d.token == "OS20-E4004")
    );
}

#[test]
fn locked_offline_from_bare_git_zero_network() {
    let cache = Arc::new(GitObjectCache::new());
    seed_git_cache(&cache, "abc123");
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        mode: OntologyResolveMode::LockedOffline,
        fetch: FetchPolicy::Never,
        manifests: standard_manifests(),
        graphs: BTreeMap::new(),
        tree: Some(cache.clone()),
        workspace_commit: Some("abc123".into()),
        lock: Some(os20_ontology_core::Os20Lockfile {
            format: 1,
            package: standard_manifests()
                .into_iter()
                .map(|m| LockedPackage {
                    name: m.name.to_string(),
                    version: m.version,
                    source: "git".into(),
                    repository: Some("https://example.com/os20/ontologies.git".into()),
                    commit: Some("abc123".into()),
                    tree: Some("treeabc".into()),
                    package_root: Some(m.package_root),
                    manifest_digest: None,
                    role: Some("ontology".into()),
                    dependencies: vec![],
                })
                .collect(),
        }),
        ..OntologyCompileRequest::default()
    });
    assert_eq!(cache.registry_call_count(), 0);
    assert_eq!(cache.git_fetch_count(), 0);
    let sdk = Os20Sdk::from_compile(r);
    assert!(
        sdk.ontology()
            .resolve_type(&tid("@os20/electrical#ElectricMachine"))
            .is_some()
    );
}

#[test]
fn online_update_hits_registry() {
    let reg = Arc::new(RecordingRegistry::new());
    reg.set_latest("@os20/physics", "0.1.0");
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        mode: OntologyResolveMode::Update,
        fetch: FetchPolicy::Allow,
        manifests: vec![physics_manifest()],
        graphs: BTreeMap::from([("@os20/physics".into(), physics_graph())]),
        lock: Some(os20_ontology_core::Os20Lockfile {
            format: 1,
            package: vec![LockedPackage {
                name: "@os20/physics".into(),
                version: "0.1.0".into(),
                source: "registry".into(),
                repository: None,
                commit: None,
                tree: None,
                package_root: Some("physics".into()),
                manifest_digest: None,
                role: Some("ontology".into()),
                dependencies: vec![],
            }],
        }),
        registry: Some(reg.clone()),
        ..OntologyCompileRequest::default()
    });
    assert!(reg.call_count() >= 1);
    assert!(
        Os20Sdk::from_compile(r)
            .ontology()
            .resolve_type(&tid("@os20/physics#Mass"))
            .is_some()
    );
}

#[test]
fn missing_source_offline() {
    let cache = Arc::new(GitObjectCache::new());
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        mode: OntologyResolveMode::LockedOffline,
        fetch: FetchPolicy::Never,
        manifests: vec![core_manifest()],
        graphs: BTreeMap::new(),
        tree: Some(cache),
        workspace_commit: Some("dead".into()),
        ..OntologyCompileRequest::default()
    });
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::MissingSourceOffline && d.token == "OS20-E4008")
    );
}

#[test]
fn delete_os20_rebuild() {
    let r = compile_standard();
    let sdk = Os20Sdk::from_compile(r);
    let dir = tempfile::tempdir().unwrap();
    let os20 = dir.path().join(".os20");
    let a = reindex(sdk.ontology().snapshot(), &os20).unwrap();
    let b = reindex(sdk.ontology().snapshot(), &os20).unwrap();
    assert_eq!(a.semantic_fingerprint, b.semantic_fingerprint);
}

#[test]
fn monorepo_multiple_packages_same_repo() {
    let cache = Arc::new(GitObjectCache::new());
    seed_git_cache(&cache, "mono");
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: standard_manifests(),
        graphs: BTreeMap::new(),
        tree: Some(cache),
        workspace_commit: Some("mono".into()),
        fetch: FetchPolicy::Never,
        mode: OntologyResolveMode::LockedOffline,
        ..OntologyCompileRequest::default()
    });
    let sdk = Os20Sdk::from_compile(r);
    assert!(sdk.ontology().package("@os20/core").is_some());
    assert!(sdk.ontology().package("@os20/physics").is_some());
    assert!(sdk.ontology().package("@os20/electrical").is_some());
}

#[test]
fn cross_package_type() {
    let sdk = Os20Sdk::from_compile(compile_standard());
    let props = sdk
        .ontology()
        .properties(&tid("@os20/mechanical#RigidBody"));
    assert!(props.iter().any(|p| match &p.ty {
        os20_ontology_core::TypeRef::Bound { id } => id.as_str() == "@os20/physics#Mass",
        _ => false,
    }));
}

#[test]
fn version_upgrade_and_major_incompatible() {
    let v1 = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![kerml_manifest(), core_manifest(), physics_manifest()],
        graphs: BTreeMap::from([
            ("@omg/kerml".into(), kerml_graph()),
            ("@os20/core".into(), core_graph()),
            ("@os20/physics".into(), physics_graph()),
        ]),
        ..OntologyCompileRequest::default()
    });
    let v2 = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![kerml_manifest(), core_manifest(), physics_v2_manifest()],
        graphs: BTreeMap::from([
            ("@omg/kerml".into(), kerml_graph()),
            ("@os20/core".into(), core_graph()),
            ("@os20/physics".into(), physics_v2_graph()),
        ]),
        ..OntologyCompileRequest::default()
    });
    let s1 = Os20Sdk::from_compile(v1);
    let s2 = Os20Sdk::from_compile(v2);
    assert!(
        s1.ontology()
            .resolve_type(&tid("@os20/physics#Mass"))
            .is_some()
    );
    assert!(
        s2.ontology()
            .resolve_type(&tid("@os20/physics#Mass"))
            .is_none()
    );
    assert!(
        s2.ontology()
            .resolve_type(&tid("@os20/physics#MassV2"))
            .is_some()
    );
    assert_ne!(s1.ontology().fingerprint(), s2.ontology().fingerprint());
}

#[test]
fn unrelated_cache_valid_when_electrical_changes() {
    let base = compile_standard();
    let mut graphs = standard_graphs();
    graphs.insert("@os20/electrical".into(), {
        let mut g = electrical_graph();
        g.elements.push(BoundElement {
            name: "BusBar".into(),
            qualified: None,
            kind: "Classifier".into(),
            visibility: Visibility::Public,
            specializes: vec!["ElectricMachine".into()],
            mixins: vec![],
            features: vec![],
            domains: vec!["Electrical".into()],
            quantity_dimension: None,
            preferred_unit: None,
            value_type: None,
            replaced_by: None,
            split_into: vec![],
            lifecycle: os20_ontology_core::LifecycleStatus::Active,
            comment: None,
            file: "Electrical.sysml".into(),
            span: os20_ontology_core::SourceSpan { start: 0, end: 1 },
        });
        g
    });
    let changed = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: standard_manifests(),
        graphs,
        ..OntologyCompileRequest::default()
    });
    let mech_before = base
        .packages
        .iter()
        .find(|p| p.package.ontology_id.as_str() == "@os20/mechanical")
        .unwrap()
        .package
        .fingerprint
        .clone();
    let mech_after = changed
        .packages
        .iter()
        .find(|p| p.package.ontology_id.as_str() == "@os20/mechanical")
        .unwrap()
        .package
        .fingerprint
        .clone();
    assert_eq!(mech_before, mech_after);
}

#[test]
fn comment_only_no_semantic_invalidation() {
    let a = compile_standard();
    let mut graphs = standard_graphs();
    let mut g = core_graph();
    g.elements[0].comment = Some("editorial".into());
    graphs.insert("@os20/core".into(), g);
    let b = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: standard_manifests(),
        graphs,
        ..OntologyCompileRequest::default()
    });
    assert!(comment_only_edit(
        a.snapshot.semantic_fingerprint(),
        b.snapshot.semantic_fingerprint()
    ));
    let delta = SourceGraphDelta {
        comment_only: true,
        changed_files: vec!["Thing.sysml".into()],
        ..SourceGraphDelta::default()
    };
    let inv = invalidate_from_delta(&a.snapshot, &b.snapshot, &delta);
    assert!(inv.semantic_noop);
}

#[test]
fn parent_type_change_invalidates_subtypes() {
    let before = compile_standard();
    let mut graphs = standard_graphs();
    let mut g = core_graph();
    for el in &mut g.elements {
        if el.name == "Machine" {
            el.specializes = vec!["Thing".into()];
        }
    }
    graphs.insert("@os20/core".into(), g);
    let after = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: standard_manifests(),
        graphs,
        ..OntologyCompileRequest::default()
    });
    let inv = invalidate_from_delta(
        &before.snapshot,
        &after.snapshot,
        &SourceGraphDelta {
            comment_only: false,
            changed_files: vec!["Machine.sysml".into()],
            ..SourceGraphDelta::default()
        },
    );
    assert!(
        inv.compatibility
            .contains(&tid("@os20/electrical#ElectricMachine"))
    );
}

#[test]
fn two_sdks_different_versions() {
    let a = Os20Sdk::from_compile(compile_standard());
    let b = Os20Sdk::bootstrap();
    assert_ne!(
        format!("{:?}", a.ontology().snapshot().source_kind()),
        format!("{:?}", b.ontology().snapshot().source_kind())
    );
}

#[test]
fn missing_required_ontology() {
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        required: vec![PackageId::new("@os20/thermodynamics").unwrap()],
        manifests: vec![],
        ..OntologyCompileRequest::default()
    });
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::MissingOntology)
    );
}

#[test]
fn protected_redefinition() {
    let mut g = electrical_graph();
    g.elements.push(BoundElement {
        name: "Thing".into(),
        qualified: Some("@os20/core#Thing".into()),
        kind: "Classifier".into(),
        visibility: Visibility::Public,
        specializes: vec![],
        mixins: vec![],
        features: vec![],
        domains: vec![],
        quantity_dimension: None,
        preferred_unit: None,
        value_type: None,
        replaced_by: None,
        split_into: vec![],
        lifecycle: os20_ontology_core::LifecycleStatus::Active,
        comment: None,
        file: "Hack.sysml".into(),
        span: os20_ontology_core::SourceSpan::default(),
    });
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![electrical_manifest()],
        graphs: BTreeMap::from([("@os20/electrical".into(), g)]),
        ..OntologyCompileRequest::default()
    });
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::ProtectedRedefinition && d.token == "OS20-E4002")
    );
}

#[test]
fn unresolved_and_evolution_missing() {
    let mut g = core_graph();
    g.elements[0].specializes.push("NoSuch".into());
    g.elements[0].replaced_by = Some("MissingReplacement".into());
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![core_manifest()],
        graphs: BTreeMap::from([("@os20/core".into(), g)]),
        ..OntologyCompileRequest::default()
    });
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::UnresolvedType && d.token == "OS20-E4003")
    );
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::EvolutionTargetMissing && d.token == "OS20-E4006")
    );
}

#[test]
fn invalid_domain() {
    let mut g = core_graph();
    g.elements[0].domains.push("".into());
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![core_manifest()],
        graphs: BTreeMap::from([("@os20/core".into(), g)]),
        ..OntologyCompileRequest::default()
    });
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::InvalidDomainDeclaration && d.token == "OS20-E4005")
    );
}

#[test]
fn malformed_source_no_panic() {
    let cache = Arc::new(GitObjectCache::new());
    cache.insert(
        "c",
        "core/src/ontology.sourcegraph.json",
        b"{not json".to_vec(),
    );
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![core_manifest()],
        graphs: BTreeMap::new(),
        tree: Some(cache),
        workspace_commit: Some("c".into()),
        fetch: FetchPolicy::Never,
        ..OntologyCompileRequest::default()
    });
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::MalformedOntologySource)
    );
}

#[test]
fn file_order_independent() {
    let mut g1 = core_graph();
    g1.elements.reverse();
    let a = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![core_manifest(), kerml_manifest()],
        graphs: BTreeMap::from([
            ("@os20/core".into(), core_graph()),
            ("@omg/kerml".into(), kerml_graph()),
        ]),
        ..OntologyCompileRequest::default()
    });
    let b = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![kerml_manifest(), core_manifest()],
        graphs: BTreeMap::from([
            ("@omg/kerml".into(), kerml_graph()),
            ("@os20/core".into(), g1),
        ]),
        ..OntologyCompileRequest::default()
    });
    assert_eq!(
        a.snapshot.semantic_fingerprint(),
        b.snapshot.semantic_fingerprint()
    );
}

#[test]
fn stdlib_and_ontology_ids_do_not_collide() {
    let sdk = Os20Sdk::from_compile(compile_standard());
    let real = sdk
        .ontology()
        .resolve_type(&tid("@omg/kerml#Real"))
        .unwrap();
    let thing = sdk
        .ontology()
        .resolve_type(&tid("@os20/core#Thing"))
        .unwrap();
    assert_ne!(real.id, thing.id);
}

#[test]
fn private_not_in_find_types_public_api() {
    let mut g = core_graph();
    g.elements.push(BoundElement {
        name: "HiddenHelper".into(),
        qualified: None,
        kind: "Classifier".into(),
        visibility: Visibility::Private,
        specializes: vec!["Thing".into()],
        mixins: vec![],
        features: vec![],
        domains: vec![],
        quantity_dimension: None,
        preferred_unit: None,
        value_type: None,
        replaced_by: None,
        split_into: vec![],
        lifecycle: os20_ontology_core::LifecycleStatus::Active,
        comment: None,
        file: "Hidden.sysml".into(),
        span: os20_ontology_core::SourceSpan::default(),
    });
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![core_manifest(), kerml_manifest()],
        graphs: BTreeMap::from([
            ("@os20/core".into(), g),
            ("@omg/kerml".into(), kerml_graph()),
        ]),
        ..OntologyCompileRequest::default()
    });
    let sdk = Os20Sdk::from_compile(r);
    let hits = sdk.ontology().find_types("HiddenHelper");
    assert!(hits.is_empty());
    assert!(
        sdk.ontology()
            .resolve_type(&tid("@os20/core#HiddenHelper"))
            .is_some()
    );
}

#[test]
fn compile_10k_types() {
    let mut elements = Vec::new();
    elements.push(BoundElement {
        name: "Thing".into(),
        qualified: None,
        kind: "Classifier".into(),
        visibility: Visibility::Public,
        specializes: vec![],
        mixins: vec![],
        features: vec![],
        domains: vec![],
        quantity_dimension: None,
        preferred_unit: None,
        value_type: None,
        replaced_by: None,
        split_into: vec![],
        lifecycle: os20_ontology_core::LifecycleStatus::Active,
        comment: None,
        file: "G.sysml".into(),
        span: os20_ontology_core::SourceSpan::default(),
    });
    for i in 0..10_000 {
        elements.push(BoundElement {
            name: format!("N{i}"),
            qualified: None,
            kind: "Classifier".into(),
            visibility: Visibility::Public,
            specializes: vec!["Thing".into()],
            mixins: vec![],
            features: vec![],
            domains: vec![],
            quantity_dimension: None,
            preferred_unit: None,
            value_type: None,
            replaced_by: None,
            split_into: vec![],
            lifecycle: os20_ontology_core::LifecycleStatus::Active,
            comment: None,
            file: "G.sysml".into(),
            span: os20_ontology_core::SourceSpan::default(),
        });
    }
    let graph = BoundSourceGraph {
        package: "@os20/core".into(),
        elements,
        relations: vec![],
        file_comments: vec![],
    };
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![core_manifest()],
        graphs: BTreeMap::from([("@os20/core".into(), graph)]),
        ..OntologyCompileRequest::default()
    });
    let sdk = Os20Sdk::from_compile(r);
    assert!(
        sdk.ontology()
            .resolve_type(&tid("@os20/core#N9999"))
            .is_some()
    );
}

#[test]
fn locked_load_warm_cold() {
    let r = compile_standard();
    let sdk = Os20Sdk::from_compile(r);
    let dir = tempfile::tempdir().unwrap();
    let cold = dir.path().join(".os20");
    let idx = reindex(sdk.ontology().snapshot(), &cold).unwrap();
    let warm = os20_ontology_core::DerivedOntologyIndex::read_from_os20_dir(&cold).unwrap();
    assert_eq!(idx.semantic_fingerprint, warm.semantic_fingerprint);
}

#[test]
fn file_move_preserves_identity() {
    let mut g = core_graph();
    g.elements[0].file = "moved/Thing.sysml".into();
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![core_manifest(), kerml_manifest()],
        graphs: BTreeMap::from([
            ("@os20/core".into(), g),
            ("@omg/kerml".into(), kerml_graph()),
        ]),
        ..OntologyCompileRequest::default()
    });
    let sdk = Os20Sdk::from_compile(r);
    let t = sdk
        .ontology()
        .resolve_type(&tid("@os20/core#Thing"))
        .unwrap();
    assert_eq!(t.id.as_str(), "@os20/core#Thing");
    assert_eq!(t.provenance.source_file, "moved/Thing.sysml");
}

#[test]
fn dependency_cycle_diagnostic() {
    let mut a = core_manifest();
    a.dependencies.push(PackageId::os20_physics());
    let mut p = physics_manifest();
    p.dependencies.push(PackageId::os20_core());
    let r = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![a, p],
        graphs: BTreeMap::from([
            ("@os20/core".into(), core_graph()),
            ("@os20/physics".into(), physics_graph()),
        ]),
        ..OntologyCompileRequest::default()
    });
    assert!(
        r.diagnostics
            .iter()
            .any(|d| d.code == DiagnosticCode::OntologyDependencyCycle && d.token == "OS20-E4010")
    );
}
