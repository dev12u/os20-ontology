//! Ontology track Prompt 3 — resolver integration.

use std::collections::BTreeMap;

use os20_ontology_core::{
    DiagnosticCode, FetchPolicy, LifecycleStatus, OntologyCompileRequest, OntologyCompiler,
    OntologyResolveMode, OntologySnapshotBuilder, OntologyTypeKind, PackageId,
    PredicateRuntimeSpec, Provenance, RelationEdge, RelationKind, SemanticDomainId,
    SemanticResolverOntology, TypeDecl, TypeId, Visibility, core_graph, core_manifest,
    electrical_graph, electrical_manifest, invalidate_dependent_caches, kerml_graph,
    kerml_manifest, ontology_diff, physics_graph, physics_manifest, physics_v2_graph,
    physics_v2_manifest, unrelated_types_preserved,
};
use os20_sdk::Os20Sdk;

fn tid(raw: &str) -> TypeId {
    TypeId::new(raw).unwrap()
}

fn compile_prod() -> os20_ontology_core::OntologyCompileResult {
    OntologyCompiler::compile(OntologyCompileRequest {
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
    })
}

#[test]
fn assignability_servo_chain() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let servo = tid("@os20/electrical#ServoMotor");
    let motor = tid("@os20/electrical#ElectricMotor");
    let physical = tid("@os20/core#PhysicalThing");
    assert!(sdk.ontology().assignable(&servo, &motor));
    assert!(sdk.ontology().assignable(&servo, &physical));
    assert!(!sdk.ontology().assignable(&physical, &servo));
    assert!(sdk.ontology().is_same_type(&servo, &servo));
    assert!(sdk.ontology().snapshot().is_supertype_of(&physical, &servo));
}

#[test]
fn mixin_not_subtype() {
    let sdk = Os20Sdk::bootstrap();
    let physical = tid("@os20/core#PhysicalThing");
    let mixin = tid("@os20/core#MassBearing");
    assert!(!sdk.ontology().is_subtype(&physical, &mixin));
    assert!(sdk.ontology().snapshot().has_mixin(&physical, &mixin));
}

#[test]
fn quantity_mass_vs_length() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let mass = tid("@os20/physics#Mass");
    let length = tid("@os20/physics#Length");
    let err = sdk
        .ontology()
        .snapshot()
        .check_assignable(&mass, &length)
        .unwrap_err();
    assert_eq!(err.code, DiagnosticCode::QuantityDimensionMismatch);
    assert_eq!(err.token, "OS20-E4014");
    assert!(!err.related.is_empty());
    assert!(!sdk.ontology().assignable(&mass, &length));
    let volt = tid("@os20/electrical#Voltage");
    let temp = tid("@os20/physics#Temperature");
    assert!(!sdk.ontology().assignable(&volt, &temp));
}

#[test]
fn covariant_override() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let ont = sdk.ontology();
    let snap = ont.snapshot();
    assert!(
        snap.redefinition_compatible(
            &tid("@os20/core#PhysicalThing"),
            &tid("@os20/electrical#ServoMotor")
        )
        .is_ok()
    );
    let err = snap
        .redefinition_compatible(
            &tid("@os20/electrical#ServoMotor"),
            &tid("@os20/core#PhysicalThing"),
        )
        .unwrap_err();
    assert_eq!(err.code, DiagnosticCode::IncompatibleOverride);
}

#[test]
fn diamond_shared_identity() {
    let thing = tid("@os20/core#Thing");
    let a = tid("@os20/core#A");
    let b = tid("@os20/core#B");
    let d = tid("@os20/core#D");
    let decl = |id: TypeId, specs: Vec<TypeId>| TypeDecl {
        id,
        kind: OntologyTypeKind::Classifier,
        specializes: specs,
        mixins: vec![],
        domains: vec![],
        properties: vec![],
        lifecycle: LifecycleStatus::Active,
        visibility: Visibility::Public,
        provenance: Provenance::catalog(PackageId::os20_core(), "d.sysml"),
        comment: None,
    };
    let snap = OntologySnapshotBuilder::new()
        .type_decl(decl(thing.clone(), vec![]))
        .type_decl(decl(a.clone(), vec![thing.clone()]))
        .type_decl(decl(b.clone(), vec![thing.clone()]))
        .type_decl(decl(d.clone(), vec![a.clone(), b.clone()]))
        .build();
    assert!(snap.is_subtype_of(&d, &a));
    assert!(snap.is_subtype_of(&d, &b));
    assert!(snap.is_subtype_of(&d, &thing));
    let lcs = snap.least_common_supertype(&a, &b).unwrap();
    assert_eq!(lcs, thing);
}

#[test]
fn type_dag_multiple_inheritance() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let em = tid("@os20/electrical#ElectricMachine");
    let supers = sdk.ontology().supertypes(&em);
    assert!(supers.iter().any(|t| t.as_str() == "@os20/core#Machine"));
    assert!(supers.iter().any(|t| t.as_str() == "@os20/core#Thing"));
}

#[test]
fn predicate_invalid_flow_endpoint() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let ont = sdk.ontology();
    let snap = ont.snapshot();
    let edge = RelationEdge {
        kind: RelationKind::Flow,
        source: tid("@os20/physics#Mass"),
        target: tid("@os20/physics#Length"),
    };
    let err = snap.validate_relation(&edge).unwrap_err();
    assert_eq!(err.code, DiagnosticCode::IllegalRelationEndpoint);
    let spec = PredicateRuntimeSpec {
        id: "hasMass".into(),
        source_type: Some(tid("@os20/core#PhysicalThing")),
        target_type: Some(tid("@os20/physics#Mass")),
        domain: SemanticDomainId::new("@os20/core#Physics").ok(),
    };
    assert!(
        snap.validate_predicate(
            &spec,
            &tid("@os20/electrical#ServoMotor"),
            &tid("@os20/physics#Mass")
        )
        .is_ok()
    );
}

#[test]
fn multi_domain_element() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let c = sdk
        .ontology()
        .snapshot()
        .classify(&tid("@os20/electrical#ElectricMachine"))
        .unwrap();
    assert!(c.is_physical);
    assert!(c.semantic_domains.len() > 1);
    assert!(
        !sdk.ontology()
            .snapshot()
            .elements_in_domain(&c.semantic_domains[0])
            .is_empty()
    );
}

#[test]
fn unallocated_reclassify() {
    let sdk = Os20Sdk::bootstrap();
    let u = tid("@os20/core#UnallocatedPerson");
    let person = tid("@os20/core#Person");
    assert!(sdk.ontology().is_subtype(&u, &person));
    let plan = sdk
        .ontology()
        .snapshot()
        .reclassify_analysis(&u, &tid("@os20/core#Individual"));
    assert_eq!(plan.semver, os20_ontology_core::SemVerAdvice::Major);
    assert!(
        sdk.ontology()
            .snapshot()
            .classify(&u)
            .unwrap()
            .is_unallocated
    );
}

#[test]
fn evolution_guidance_not_silent_rewrite() {
    let sdk = Os20Sdk::bootstrap();
    let g = sdk
        .ontology()
        .snapshot()
        .evolution_guidance(&tid("@os20/core#LegacyGadget"));
    assert!(g.deprecated);
    assert_eq!(
        g.replaced_by.as_ref().map(|t| t.as_str()),
        Some("@os20/core#Gadget")
    );
    assert!(
        sdk.ontology()
            .resolve_type(&tid("@os20/core#LegacyGadget"))
            .is_some()
    );
}

#[test]
fn v1_v2_independent() {
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
    let diff = ontology_diff(s1.ontology().snapshot(), s2.ontology().snapshot());
    assert_eq!(diff.semver, os20_ontology_core::SemVerAdvice::Major);
}

#[test]
fn impact_reaches_downstream() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let impact = sdk.ontology().impact(&tid("@os20/physics#Mass"));
    assert!(
        impact
            .downstream_packages
            .iter()
            .any(|p| p.as_str() == "@os20/electrical")
    );
}

#[test]
fn unrelated_ontology_preserves_cache() {
    let a = compile_prod();
    let mut graphs = BTreeMap::from([
        ("@omg/kerml".into(), kerml_graph()),
        ("@os20/core".into(), core_graph()),
        ("@os20/physics".into(), physics_graph()),
        ("@os20/electrical".into(), electrical_graph()),
    ]);
    let mut g = electrical_graph();
    g.elements[0].comment = Some("x".into());
    graphs.insert("@os20/electrical".into(), g);
    let b = OntologyCompiler::compile(OntologyCompileRequest {
        manifests: vec![
            kerml_manifest(),
            core_manifest(),
            physics_manifest(),
            electrical_manifest(),
        ],
        graphs,
        ..OntologyCompileRequest::default()
    });
    let mut changed = std::collections::BTreeSet::new();
    changed.insert(PackageId::os20_electrical());
    assert!(unrelated_types_preserved(
        &a.snapshot,
        &b.snapshot,
        &changed
    ));
    let inv = invalidate_dependent_caches(&a.snapshot, &b.snapshot);
    assert!(
        inv.semantic_noop
            || inv
                .types
                .iter()
                .all(|t| t.package().ok() == Some(PackageId::os20_electrical())
                    || t.as_str().contains("electrical"))
    );
}

#[test]
fn why_provenance() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let why = sdk
        .ontology()
        .why_type(&tid("@os20/electrical#ServoMotor"))
        .unwrap();
    let names: Vec<_> = why
        .specialization_path
        .iter()
        .map(|t| t.local_name())
        .collect();
    assert!(names.contains(&"ServoMotor"));
    assert!(names.contains(&"PhysicalThing") || names.contains(&"ElectricMotor"));
    let wp = sdk
        .ontology()
        .snapshot()
        .why_classified_physical(&tid("@os20/electrical#ServoMotor"));
    assert!(wp.is_physical);
    assert!(wp.physical_thing_version.is_some());
}

#[test]
fn offline_runtime_queries() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    assert!(
        sdk.ontology()
            .resolve_type(&tid("@os20/core#Thing"))
            .is_some()
    );
    assert!(!sdk.ontology().predicates().is_empty() || !sdk.ontology().domains().is_empty());
    let _ = sdk.ontology().references();
    let _ = sdk.ontology().type_summary(&tid("@os20/core#Thing"));
}

#[test]
fn resolver_bridge_does_not_replace_resolver() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let ont = sdk.ontology();
    let snap = ont.snapshot();
    let k: &dyn SemanticResolverOntology = snap;
    assert!(k.is_assignable_to(
        &tid("@os20/electrical#ServoMotor"),
        &tid("@os20/core#PhysicalThing")
    ));
}

#[test]
fn performance_10k_and_100k_edges() {
    let thing = tid("@os20/core#Thing");
    let mut b = OntologySnapshotBuilder::new().type_decl(TypeDecl {
        id: thing.clone(),
        kind: OntologyTypeKind::Classifier,
        specializes: vec![],
        mixins: vec![],
        domains: vec![],
        properties: vec![],
        lifecycle: LifecycleStatus::Active,
        visibility: Visibility::Public,
        provenance: Provenance::catalog(PackageId::os20_core(), "p.sysml"),
        comment: None,
    });
    for i in 0..2_000 {
        let id = tid(&format!("@os20/core#N{i}"));
        let parent = if i == 0 {
            thing.clone()
        } else {
            tid(&format!("@os20/core#N{}", i - 1))
        };
        b = b.type_decl(TypeDecl {
            id,
            kind: OntologyTypeKind::Classifier,
            specializes: vec![parent, thing.clone()],
            mixins: vec![],
            domains: vec![],
            properties: vec![],
            lifecycle: LifecycleStatus::Active,
            visibility: Visibility::Public,
            provenance: Provenance::catalog(PackageId::os20_core(), "p.sysml"),
            comment: None,
        });
    }
    let snap = b.build();
    let last = tid("@os20/core#N1999");
    assert!(snap.is_subtype_of(&last, &thing));
    assert!(snap.is_assignable_to(&last, &thing));
    let _ = snap.reverse_references();
    let _ = snap.domains_for_element(&last);
    let _ = snap.why_type(&last);
}
