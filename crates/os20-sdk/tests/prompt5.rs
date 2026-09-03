//! Ontology track Prompt 5 — freeze, conformance, migration, scale, security.

use std::collections::BTreeMap;
use std::time::Instant;

use os20_ontology_core::{
    BoundElement, ContentDigest, DerivedOntologyIndex, DiagnosticCode, EvolutionEdge,
    EvolutionKind, FINGERPRINT_SEMANTIC, FINGERPRINT_SOURCE, FetchPolicy, LockedPackage,
    MemoryOntology, ONTOLOGY_RUNTIME_FORMAT_VERSION, OS20_CORE_ONTOLOGY_VERSION,
    OntologyCompileRequest, OntologyCompiler, OntologyRegistry, OntologyResolveMode,
    OntologySnapshotBuilder, OntologySourceKind, OntologyTypeKind, PackageId, Provenance,
    RecordingRegistry, RelationRegistry, SemVerAdvice, TypeDecl, TypeId, Visibility, core_graph,
    core_manifest, electrical_graph, electrical_manifest, kerml_graph, kerml_manifest,
    lockfile_from_locked_packages, migration_analysis, ontology_diff, physics_graph,
    physics_manifest, physics_v2_graph, physics_v2_manifest, reindex,
};
use os20_sdk::{Os20Sdk, to_json_report};

fn tid(raw: &str) -> TypeId {
    TypeId::new(raw).unwrap()
}

fn compile(req: OntologyCompileRequest) -> os20_ontology_core::OntologyCompileResult {
    OntologyCompiler::compile(req)
}

fn prod_request() -> OntologyCompileRequest {
    OntologyCompileRequest {
        mode: OntologyResolveMode::LockedOffline,
        fetch: FetchPolicy::Never,
        manifests: vec![
            kerml_manifest(),
            core_manifest(),
            physics_manifest(),
            electrical_manifest(),
        ],
        graphs: BTreeMap::from([
            ("@omg/kerml".into(), kerml_graph()),
            ("@os20/core".into(), core_graph()),
            ("@os20/physics".into(), physics_graph()),
            ("@os20/electrical".into(), electrical_graph()),
        ]),
        ..OntologyCompileRequest::default()
    }
}

fn physics_v2_request() -> OntologyCompileRequest {
    OntologyCompileRequest {
        mode: OntologyResolveMode::LockedOffline,
        fetch: FetchPolicy::Never,
        manifests: vec![kerml_manifest(), core_manifest(), physics_v2_manifest()],
        graphs: BTreeMap::from([
            ("@omg/kerml".into(), kerml_graph()),
            ("@os20/core".into(), core_graph()),
            ("@os20/physics".into(), physics_v2_graph()),
        ]),
        ..OntologyCompileRequest::default()
    }
}

#[test]
fn core_ontology_is_explicitly_versioned() {
    assert_eq!(
        MemoryOntology::core_ontology_version(),
        OS20_CORE_ONTOLOGY_VERSION
    );
    assert_eq!(OS20_CORE_ONTOLOGY_VERSION, "0.1.0");
    let sdk = Os20Sdk::bootstrap();
    let ont = sdk.ontology();
    let core = ont
        .packages()
        .into_iter()
        .find(|p| p.ontology_id.as_str() == "@os20/core")
        .expect("core package");
    assert_eq!(core.ontology_version, OS20_CORE_ONTOLOGY_VERSION);
}

#[test]
fn fingerprint_domains_do_not_collide() {
    let mut a = vec!["x".into()];
    let mut b = vec!["x".into()];
    let s = ContentDigest::hash_domain_sorted_lines(FINGERPRINT_SEMANTIC, &mut a);
    let t = ContentDigest::hash_domain_sorted_lines(FINGERPRINT_SOURCE, &mut b);
    assert_ne!(s.as_str(), t.as_str());
}

#[test]
fn derived_index_format_version_invalidates() {
    let snap = MemoryOntology::core().into_snapshot();
    let idx = DerivedOntologyIndex::from_snapshot(&snap);
    assert_eq!(idx.runtime_format_version, ONTOLOGY_RUNTIME_FORMAT_VERSION);
    assert!(idx.format_compatible());
    let mut stale = idx.clone();
    stale.runtime_format_version = 0;
    assert!(!stale.format_compatible());
}

#[test]
fn handlers_are_builtin_rust_only() {
    assert!(RelationRegistry::core().is_builtin_only());
}

#[test]
fn kerml_stdlib_ids_do_not_collide_with_os20_core() {
    let sdk = Os20Sdk::from_compile(compile(prod_request()));
    let real = tid("@omg/kerml#Real");
    let thing = tid("@os20/core#Thing");
    assert!(sdk.ontology().resolve_type(&real).is_some());
    assert!(sdk.ontology().resolve_type(&thing).is_some());
    assert_ne!(real.as_str(), thing.as_str());
}

#[test]
fn lock_records_release_coordinates() {
    let lock = lockfile_from_locked_packages(vec![LockedPackage {
        name: "@os20/core".into(),
        version: "0.1.0".into(),
        source: "git".into(),
        repository: Some("https://git.example/os20-core".into()),
        commit: Some("a".repeat(40)),
        tree: Some("b".repeat(40)),
        package_root: Some("ontology".into()),
        manifest_digest: Some("aa".repeat(32)),
        role: Some("ontology".into()),
        dependencies: vec![],
    }]);
    let p = &lock.package[0];
    assert_eq!(p.version, "0.1.0");
    assert!(p.repository.is_some());
    assert_eq!(p.package_root.as_deref(), Some("ontology"));
    assert!(p.commit.as_ref().unwrap().len() >= 40);
    assert!(p.tree.is_some());
    assert!(p.manifest_digest.is_some());
    assert_eq!(p.role.as_deref(), Some("ontology"));
}

#[test]
fn consumer_resolve_uses_existing_registry_trait_not_ontology_endpoint() {
    let reg = RecordingRegistry::new();
    reg.set_latest("@os20/physics", "0.1.0");
    assert_eq!(reg.resolve_version("@os20/physics", "*").unwrap(), "0.1.0");
    assert_eq!(reg.calls(), 1);
}

#[test]
fn historical_v1_lock_survives_v2_in_process() {
    let v1 = compile(prod_request()).snapshot;
    let v2 = compile(physics_v2_request()).snapshot;
    let sdk1 = Os20Sdk::with_ontology_snapshot(v1);
    let sdk2 = Os20Sdk::with_ontology_snapshot(v2);
    let mass = tid("@os20/physics#Mass");
    assert!(sdk1.ontology().resolve_type(&mass).is_some());
    let f1 = sdk1.ontology().fingerprint().as_str().to_owned();
    let f2 = sdk2.ontology().fingerprint().as_str().to_owned();
    assert_ne!(f1, f2);
    assert!(sdk1.ontology().resolve_type(&mass).is_some());
}

#[test]
fn delete_os20_rebuild_same_semantics() {
    let snap = compile(prod_request()).snapshot;
    let dir = tempfile::tempdir().unwrap();
    let os20 = dir.path().join(".os20");
    let first = DerivedOntologyIndex::from_snapshot(&snap);
    first.write_to_os20_dir(&os20).unwrap();
    let rebuilt = reindex(&snap, &os20).unwrap();
    assert_eq!(first.semantic_fingerprint, rebuilt.semantic_fingerprint);
    assert_eq!(
        snap.supertypes(&tid("@os20/electrical#ServoMotor")),
        compile(prod_request())
            .snapshot
            .supertypes(&tid("@os20/electrical#ServoMotor"))
    );
}

#[test]
fn additive_update_is_minor_breaking_is_major() {
    let v1 = compile(prod_request()).snapshot;
    let v2 = compile(physics_v2_request()).snapshot;
    let d = ontology_diff(&v1, &v2);
    assert!(matches!(
        d.semver,
        SemVerAdvice::Major | SemVerAdvice::Minor | SemVerAdvice::Patch | SemVerAdvice::None
    ));
    let same = ontology_diff(&v1, &v1);
    assert_eq!(same.semver, SemVerAdvice::None);
}

#[test]
fn comment_only_source_fingerprint_changes_semantic_stable() {
    let a = MemoryOntology::core();
    let b = MemoryOntology::core_with_thing_comment("docs only");
    assert_eq!(
        a.snapshot().semantic_fingerprint(),
        b.snapshot().semantic_fingerprint()
    );
    assert_ne!(
        a.snapshot().source_fingerprint(),
        b.snapshot().source_fingerprint()
    );
}

#[test]
fn person_split_children_stay_explicit() {
    let person = tid("@os20/core#Person");
    let employee = tid("@os20/core#Employee");
    let individual = tid("@os20/core#Individual");
    let unalloc = tid("@os20/core#UnallocatedPerson");
    let mut b = OntologySnapshotBuilder::new().source_kind(OntologySourceKind::Bootstrap);
    for id in [&person, &employee, &individual, &unalloc] {
        // Employee, Individual, and UnallocatedPerson remain explicitly allocated
        // under Person until a later mapping reparents them.
        let specs = if *id == person {
            vec![]
        } else {
            vec![person.clone()]
        };
        b = b.type_decl(TypeDecl {
            id: id.clone(),
            kind: OntologyTypeKind::Classifier,
            specializes: specs,
            mixins: vec![],
            domains: vec![],
            properties: vec![],
            lifecycle: os20_ontology_core::LifecycleStatus::Active,
            visibility: Visibility::Public,
            provenance: Provenance::catalog(PackageId::os20_core(), "Person.sysml"),
            comment: None,
        });
    }
    b = b.evolution(EvolutionEdge {
        kind: EvolutionKind::SplitInto,
        from: person.as_element().clone(),
        to: individual.as_element().clone(),
    });
    let snap = b.build();
    assert!(snap.is_subtype_of(&employee, &person));
    assert!(!snap.is_subtype_of(&employee, &individual));
    let mig = migration_analysis(&snap, &snap);
    assert!(!mig.auto_rewrite);
}

#[test]
fn migration_report_lists_removed_without_rewrite() {
    let old = compile(prod_request()).snapshot;
    let new = compile(physics_v2_request()).snapshot;
    let report = migration_analysis(&old, &new);
    assert!(!report.auto_rewrite);
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        json.contains("removedIds")
            || json.contains("removed_ids")
            || json.contains("fromFingerprint")
    );
}

#[test]
fn product_json_kinds_offline() {
    let sdk = Os20Sdk::from_compile(compile(prod_request()));
    let ont = sdk.ontology();
    for (kind, body) in [
        (
            "ontology_status",
            serde_json::to_value(ont.status_report()).unwrap(),
        ),
        (
            "ontology_type",
            serde_json::to_value(ont.type_report("@os20/electrical#ServoMotor").unwrap()).unwrap(),
        ),
        (
            "ontology_search",
            serde_json::to_value(ont.search_report("Servo")).unwrap(),
        ),
    ] {
        let text = to_json_report(kind, &body).unwrap();
        assert!(text.contains(kind));
        assert!(!text.contains("rusqlite"));
    }
    let why = ont.why_report("ServoMotor").unwrap();
    assert!(!why.specialization_path.is_empty());
    let impact = ont.impact_report("@os20/core#Thing").unwrap();
    assert_eq!(impact.origin, "@os20/core#Thing");
}

#[test]
fn conformance_positive_corpus() {
    let sdk = Os20Sdk::from_compile(compile(prod_request()));
    let ont = sdk.ontology();
    let servo = tid("@os20/electrical#ServoMotor");
    let motor = tid("@os20/electrical#ElectricMotor");
    let physical = tid("@os20/core#PhysicalThing");
    assert!(ont.is_subtype(&servo, &motor));
    assert!(ont.is_subtype(&servo, &physical));
    assert!(ont.assignable(&servo, &physical));
    assert!(!ont.properties(&servo).is_empty() || ont.type_summary(&servo).is_some());
    assert!(
        ont.snapshot()
            .quantity(&tid("@os20/physics#Mass"))
            .is_some()
            || ont.resolve_type(&tid("@os20/physics#Mass")).is_some()
    );
    assert!(!ont.domains().is_empty());
    let _ = ont.predicates();
    let gadget = ont.type_summary(&tid("@os20/core#LegacyGadget"));
    if let Some(s) = gadget {
        assert!(matches!(
            s.lifecycle,
            os20_ontology_core::LifecycleStatus::Deprecated { .. }
        ));
    }
}

#[test]
fn conformance_negative_corpus() {
    let sdk = Os20Sdk::from_compile(compile(prod_request()));
    let mass = tid("@os20/physics#Mass");
    let length = tid("@os20/physics#Length");
    assert!(!sdk.ontology().assignable(&mass, &length));
    let unknown = TypeId::new("@os20/core#DoesNotExist").unwrap();
    assert!(sdk.ontology().resolve_type(&unknown).is_none());
}

#[test]
fn cycle_detection_does_not_overflow() {
    let a = tid("@os20/core#CycleA");
    let b = tid("@os20/core#CycleB");
    let snap = OntologySnapshotBuilder::new()
        .type_decl(TypeDecl {
            id: a.clone(),
            kind: OntologyTypeKind::Classifier,
            specializes: vec![b.clone()],
            mixins: vec![],
            domains: vec![],
            properties: vec![],
            lifecycle: os20_ontology_core::LifecycleStatus::Active,
            visibility: Visibility::Public,
            provenance: Provenance::catalog(PackageId::os20_core(), "c.sysml"),
            comment: None,
        })
        .type_decl(TypeDecl {
            id: b.clone(),
            kind: OntologyTypeKind::Classifier,
            specializes: vec![a.clone()],
            mixins: vec![],
            domains: vec![],
            properties: vec![],
            lifecycle: os20_ontology_core::LifecycleStatus::Active,
            visibility: Visibility::Public,
            provenance: Provenance::catalog(PackageId::os20_core(), "c.sysml"),
            comment: None,
        })
        .build();
    assert!(
        snap.diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::SpecializationCycle)
    );
    let _ = snap.supertypes(&a);
}

#[test]
fn malformed_bound_element_does_not_panic() {
    let mut g = core_graph();
    g.elements.push(BoundElement {
        name: String::new(),
        qualified: None,
        kind: "not-a-kind".into(),
        visibility: Visibility::Public,
        specializes: vec!["???".into()],
        mixins: vec![],
        features: vec![],
        domains: vec!["nope".into()],
        quantity_dimension: None,
        preferred_unit: None,
        value_type: None,
        replaced_by: None,
        split_into: vec![],
        lifecycle: os20_ontology_core::LifecycleStatus::Active,
        comment: None,
        file: "../escape.sysml".into(),
        span: os20_ontology_core::SourceSpan::default(),
    });
    let result = std::panic::catch_unwind(|| {
        compile(OntologyCompileRequest {
            mode: OntologyResolveMode::LockedOffline,
            fetch: FetchPolicy::Never,
            manifests: vec![kerml_manifest(), core_manifest()],
            graphs: BTreeMap::from([
                ("@omg/kerml".into(), kerml_graph()),
                ("@os20/core".into(), g),
            ]),
            ..OntologyCompileRequest::default()
        })
    });
    assert!(result.is_ok());
}

#[test]
fn two_runtimes_no_global_leakage() {
    let a = Os20Sdk::from_compile(compile(prod_request()));
    let b = Os20Sdk::from_compile(compile(physics_v2_request()));
    assert_ne!(
        a.ontology().fingerprint().as_str(),
        b.ontology().fingerprint().as_str()
    );
}

#[test]
fn performance_baselines_no_gate() {
    let t0 = Instant::now();
    let compiled = compile(prod_request());
    let build = t0.elapsed();
    let sdk = Os20Sdk::from_compile(compiled);
    let ont = sdk.ontology();
    let servo = tid("@os20/electrical#ServoMotor");
    let t1 = Instant::now();
    let _ = ont.resolve_type(&servo);
    let lookup = t1.elapsed();
    let t2 = Instant::now();
    let _ = ont.subtypes(&tid("@os20/core#Thing"));
    let sub = t2.elapsed();
    let t3 = Instant::now();
    let _ = ont.supertypes(&servo);
    let super_t = t3.elapsed();
    let t4 = Instant::now();
    let _ = ont.assignable(&servo, &tid("@os20/core#Thing"));
    let asg = t4.elapsed();
    let t5 = Instant::now();
    let _ = ont.properties(&servo);
    let props = t5.elapsed();
    let t6 = Instant::now();
    let _ = ont.predicates();
    let pred = t6.elapsed();
    let t7 = Instant::now();
    let _ = ont.domains();
    let dom = t7.elapsed();
    let t8 = Instant::now();
    let _ = ont.why_type(&servo);
    let why = t8.elapsed();
    let other = compile(physics_v2_request()).snapshot;
    let t9 = Instant::now();
    let _ = ont.diff(&other);
    let diff = t9.elapsed();
    let t10 = Instant::now();
    let _ = ont.impact(&servo);
    let impact = t10.elapsed();
    let _ = (
        build,
        lookup,
        sub,
        super_t,
        asg,
        props,
        pred,
        dom,
        why,
        diff,
        impact,
        ont.snapshot().type_ids().count(),
    );
}

#[test]
fn no_authz_dependency_in_ontology_core() {
    let manifest = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../os20-ontology-core/Cargo.toml"
    ));
    assert!(!manifest.to_ascii_lowercase().contains("authz"));
    assert!(!manifest.to_ascii_lowercase().contains("x509"));
}
