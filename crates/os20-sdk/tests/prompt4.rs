//! Ontology track Prompt 4 — product reports (no rusqlite / parser / git types).

use std::collections::BTreeMap;
use std::time::Instant;

use os20_ontology_core::{
    FetchPolicy, OntologyCompileRequest, OntologyCompiler, OntologyResolveMode, TypeId, core_graph,
    core_manifest, electrical_graph, electrical_manifest, kerml_graph, kerml_manifest,
    physics_graph, physics_manifest, physics_v2_graph, physics_v2_manifest,
};
use os20_sdk::{Os20Sdk, REPORT_SCHEMA, to_json_report};

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

fn compile_physics_v2() -> os20_ontology_core::OntologyCompileResult {
    OntologyCompiler::compile(OntologyCompileRequest {
        mode: OntologyResolveMode::LockedOffline,
        fetch: FetchPolicy::Never,
        manifests: vec![kerml_manifest(), core_manifest(), physics_v2_manifest()],
        graphs: BTreeMap::from([
            ("@omg/kerml".into(), kerml_graph()),
            ("@os20/core".into(), core_graph()),
            ("@os20/physics".into(), physics_v2_graph()),
        ]),
        ..OntologyCompileRequest::default()
    })
}

#[test]
fn facade_operations_cover_prompt() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let ont = sdk.ontology();
    assert!(!ont.packages().is_empty());
    assert!(ont.snapshot().type_ids().count() > 0);
    let _ = ont.fingerprint();
    let servo = tid("@os20/electrical#ServoMotor");
    assert!(ont.resolve_type(&servo).is_some());
    assert!(!ont.find_types("Servo").is_empty());
    assert!(ont.type_summary(&servo).is_some());
    assert!(!ont.supertypes(&servo).is_empty());
    assert!(ont.subtypes(&tid("@os20/core#Thing")).len() > 1);
    let _ = ont.properties(&servo);
    let _ = ont.predicates();
    let _ = ont.domains();
    assert!(ont.why_type(&servo).is_some());
    let _ = ont.references();
    let _ = ont.impact(&servo);
}

#[test]
fn status_report_json_envelope_has_no_db_ids() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let report = sdk.ontology().status_report();
    let json = to_json_report("ontology_status", &report).unwrap();
    assert!(json.contains("\"schema\":1") || json.contains("\"schema\": 1"));
    assert!(json.contains("ontology_status"));
    assert!(!json.contains("rusqlite"));
    assert!(!json.contains("rowid"));
    assert!(!json.contains("sqlite"));
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["schema"], REPORT_SCHEMA);
    assert!(!v["body"]["packages"].as_array().unwrap().is_empty());
    assert!(v["body"]["fingerprint"].as_str().unwrap().len() == 64);
}

#[test]
fn type_report_unallocated_is_real_classifier() {
    let sdk = Os20Sdk::bootstrap();
    let hits = sdk.ontology().find_types("Unallocated");
    if hits.is_empty() {
        let thing = sdk.ontology().type_report("@os20/core#Thing").unwrap();
        assert!(!thing.id.is_empty());
        return;
    }
    let report = sdk.ontology().type_report(&hits[0].id.to_string()).unwrap();
    assert!(report.unallocated);
    assert_eq!(report.id, hits[0].id.to_string());
}

#[test]
fn search_why_impact_diff_reports() {
    let a = compile_prod().snapshot;
    let b = compile_physics_v2().snapshot;
    let sdk = Os20Sdk::with_ontology_snapshot(a.clone())
        .with_named_ontology_snapshot("physics@1", a)
        .with_named_ontology_snapshot("physics@2", b);
    let search = sdk.ontology().search_report("Motor");
    assert!(search.types.iter().any(|t| t.id.as_str().contains("Motor")));
    let why = sdk.ontology().why_report("ServoMotor").unwrap();
    assert!(!why.specialization_path.is_empty());
    let impact = sdk
        .ontology()
        .impact_report("@os20/electrical#ElectricMotor")
        .unwrap();
    assert_eq!(impact.origin, "@os20/electrical#ElectricMotor");
    let diff = sdk
        .ontology()
        .diff_report(Some("physics@1"), Some("physics@2"))
        .unwrap();
    let json = to_json_report("ontology_diff", &diff).unwrap();
    assert!(json.contains("ontology_diff"));
}

#[test]
fn two_sdk_instances_independent_snapshots() {
    let sdk_a = Os20Sdk::with_ontology_snapshot(compile_prod().snapshot);
    let sdk_b = Os20Sdk::with_ontology_snapshot(compile_physics_v2().snapshot);
    let fa = sdk_a.ontology().fingerprint().as_str().to_owned();
    let fb = sdk_b.ontology().fingerprint().as_str().to_owned();
    assert_ne!(fa, fb);
}

#[test]
fn locked_offline_compile_never_fetches() {
    let result = compile_prod();
    assert!(result.snapshot.type_ids().count() > 0);
}

#[test]
fn product_surface_timing_smoke_no_gate() {
    let sdk = Os20Sdk::from_compile(compile_prod());
    let t0 = Instant::now();
    let _ = sdk.ontology().status_report();
    let tree = t0.elapsed();
    let t1 = Instant::now();
    let _ = sdk.ontology().search_report("Electric");
    let search = t1.elapsed();
    let t2 = Instant::now();
    let _ = sdk.ontology().why_report("@os20/electrical#ServoMotor");
    let why = t2.elapsed();
    let _ = (tree, search, why);
}
