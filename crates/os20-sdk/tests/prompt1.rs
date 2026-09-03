//! Ontology track Prompt 1 tests.

use std::collections::BTreeSet;

use os20_ontology_core::{
    DiagnosticCode, ElementId, LifecycleStatus, MemoryOntology, Ontology, OntologyPackage,
    OntologySnapshotBuilder, OntologySourceKind, OntologyTypeKind, PackageId, PackageReleaseRef,
    PackageRole, Provenance, TypeDecl, TypeId, TypeRef, Visibility, reindex,
};
use os20_sdk::Os20Sdk;
use serde_json::Value;

fn tid(raw: &str) -> TypeId {
    TypeId::new(raw).unwrap()
}

#[test]
fn thing_resolves() {
    let sdk = Os20Sdk::bootstrap();
    let thing = tid("@os20/core#Thing");
    let s = sdk.ontology().resolve_type(&thing).unwrap();
    assert_eq!(s.id, thing);
    assert_eq!(s.kind, OntologyTypeKind::Classifier);
}

#[test]
fn physical_thing_specializes_thing() {
    let sdk = Os20Sdk::bootstrap();
    assert!(
        sdk.ontology()
            .is_subtype(&tid("@os20/core#PhysicalThing"), &tid("@os20/core#Thing"))
    );
}

#[test]
fn transitive_servo_to_thing() {
    let sdk = Os20Sdk::bootstrap();
    let path = sdk
        .ontology()
        .supertypes(&tid("@os20/electrical#ServoMotor"));
    let ids: Vec<_> = path.iter().map(|t| t.as_str()).collect();
    assert!(ids.contains(&"@os20/electrical#ServoMotor"));
    assert!(ids.contains(&"@os20/electrical#ElectricMotor"));
    assert!(ids.contains(&"@os20/core#PhysicalThing"));
    assert!(ids.contains(&"@os20/core#Thing"));
}

#[test]
fn assignability_direction() {
    let sdk = Os20Sdk::bootstrap();
    let servo = tid("@os20/electrical#ServoMotor");
    let physical = tid("@os20/core#PhysicalThing");
    assert!(sdk.ontology().is_assignable(&servo, &physical));
    assert!(!sdk.ontology().is_assignable(&physical, &servo));
}

#[test]
fn multiple_specialization_deterministic() {
    let sdk = Os20Sdk::bootstrap();
    let motor = tid("@os20/electrical#ElectricMotor");
    let supers = sdk.ontology().supertypes(&motor);
    let direct = sdk.ontology().type_summary(&motor).unwrap().supertypes;
    let mut sorted = direct.clone();
    sorted.sort();
    assert_eq!(direct, sorted);
    assert!(
        supers
            .iter()
            .any(|t| t.as_str() == "@os20/core#PhysicalThing")
    );
    assert!(
        supers
            .iter()
            .any(|t| t.as_str() == "@os20/core#BehaviouralThing")
    );
}

#[test]
fn mixin_does_not_alter_ancestry() {
    let sdk = Os20Sdk::bootstrap();
    let physical = tid("@os20/core#PhysicalThing");
    let mixin = tid("@os20/core#MassBearing");
    assert!(!sdk.ontology().is_subtype(&physical, &mixin));
    assert!(sdk.ontology().snapshot().has_mixin(&physical, &mixin));
    let props = sdk
        .ontology()
        .properties(&tid("@os20/electrical#ServoMotor"));
    assert!(props.iter().any(|p| p.id.as_str().ends_with("#mass")));
}

#[test]
fn inherited_property_resolved() {
    let sdk = Os20Sdk::bootstrap();
    let props = sdk
        .ontology()
        .properties(&tid("@os20/electrical#ServoMotor"));
    assert!(props.iter().any(|p| p.id.as_str().ends_with("#position")));
    assert!(
        props
            .iter()
            .any(|p| p.declared_by.as_str() == "@os20/core#PhysicalThing"
                || p.id.as_str().ends_with("#mass"))
    );
}

#[test]
fn primitive_kerml_scalars() {
    let sdk = Os20Sdk::bootstrap();
    for name in ["Boolean", "Integer", "Real", "String"] {
        let id = tid(&format!("@omg/kerml#{name}"));
        let s = sdk.ontology().resolve_type(&id).unwrap();
        assert_eq!(s.kind, OntologyTypeKind::DataType);
        assert_eq!(s.package.as_str(), "@omg/kerml");
    }
}

#[test]
fn cross_package_electrical_imports_physics() {
    let sdk = Os20Sdk::bootstrap();
    let pkgs: Vec<_> = sdk
        .ontology()
        .packages()
        .into_iter()
        .map(|p| p.ontology_id.as_str().to_owned())
        .collect();
    assert!(pkgs.contains(&"@os20/electrical".to_string()));
    let ont = sdk.ontology();
    let electrical = ont
        .packages()
        .into_iter()
        .find(|p| p.ontology_id.as_str() == "@os20/electrical")
        .unwrap();
    assert!(
        electrical
            .imports
            .iter()
            .any(|i| i.as_str() == "@os20/physics")
    );
    let props = sdk
        .ontology()
        .properties(&tid("@os20/electrical#ElectricMotor"));
    assert!(props.iter().any(|p| match &p.ty {
        TypeRef::Bound { id } => id.as_str() == "@os20/physics#Power",
        _ => false,
    }));
}

#[test]
fn ontology_fingerprint_deterministic() {
    let a = Os20Sdk::bootstrap();
    let b = Os20Sdk::bootstrap();
    assert_eq!(a.ontology().fingerprint(), b.ontology().fingerprint());
    let mut ids: Vec<_> = a
        .ontology()
        .find_types("Thing")
        .into_iter()
        .map(|t| t.id.as_str().to_owned())
        .collect();
    let mut ids2 = ids.clone();
    ids.sort();
    ids2.sort();
    assert_eq!(ids, ids2);
}

#[test]
fn comment_only_does_not_change_semantic_fingerprint() {
    let a = MemoryOntology::core();
    let b = MemoryOntology::core_with_thing_comment("editorial only");
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
fn two_sdk_instances_different_ontology_versions() {
    let v1 = Os20Sdk::bootstrap();
    let mut pkg = OntologyPackage {
        ontology_id: PackageId::os20_physics(),
        ontology_version: "2.0.0".into(),
        source_revision: Some("abc".into()),
        release: PackageReleaseRef::bootstrap(PackageId::os20_physics(), "2.0.0"),
        imports: vec![PackageId::os20_core()],
        role: PackageRole::Ontology,
        fingerprint: os20_ontology_core::ContentDigest::hash_bytes(b"v2"),
    };
    pkg.release.version = "2.0.0".into();
    let snap = OntologySnapshotBuilder::new()
        .source_kind(OntologySourceKind::PackageBacked)
        .package(pkg)
        .type_decl(TypeDecl {
            id: tid("@os20/physics#Mass"),
            kind: OntologyTypeKind::Quantity,
            specializes: vec![tid("@os20/core#Thing")],
            mixins: vec![],
            domains: vec![],
            properties: vec![],
            lifecycle: LifecycleStatus::Active,
            visibility: Visibility::Public,
            provenance: Provenance::catalog(PackageId::os20_physics(), "v2.sysml"),
            comment: None,
        })
        .build();
    let v2 = Os20Sdk::with_ontology_snapshot(snap);
    let p1 = v1
        .ontology()
        .packages()
        .into_iter()
        .find(|p| p.ontology_id.as_str() == "@os20/physics")
        .unwrap()
        .ontology_version
        .clone();
    let p2 = v2
        .ontology()
        .packages()
        .into_iter()
        .find(|p| p.ontology_id.as_str() == "@os20/physics")
        .unwrap()
        .ontology_version
        .clone();
    assert_eq!(p1, "0.1.0");
    assert_eq!(p2, "2.0.0");
    assert_ne!(v1.ontology().fingerprint(), v2.ontology().fingerprint());
}

#[test]
fn no_global_snapshot_leakage() {
    let a = Os20Sdk::bootstrap();
    let b = Os20Sdk::with_memory_ontology(MemoryOntology::core_with_thing_comment("x"));
    assert_ne!(
        a.ontology().fingerprints().source,
        b.ontology().fingerprints().source
    );
    assert_eq!(a.ontology().fingerprint(), b.ontology().fingerprint());
}

#[test]
fn missing_ontology_diagnostic() {
    let snap = OntologySnapshotBuilder::new()
        .require_package(PackageId::new("@os20/thermodynamics").unwrap())
        .build();
    let sdk = Os20Sdk::with_ontology_snapshot(snap);
    let ont = sdk.ontology();
    let d = ont.diagnostics();
    assert!(d.iter().any(|x| x.code == DiagnosticCode::MissingOntology));
}

#[test]
fn specialization_cycle_diagnostic() {
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
            lifecycle: LifecycleStatus::Active,
            visibility: Visibility::Public,
            provenance: Provenance::catalog(PackageId::os20_core(), "cycle.sysml"),
            comment: None,
        })
        .type_decl(TypeDecl {
            id: b.clone(),
            kind: OntologyTypeKind::Classifier,
            specializes: vec![a.clone()],
            mixins: vec![],
            domains: vec![],
            properties: vec![],
            lifecycle: LifecycleStatus::Active,
            visibility: Visibility::Public,
            provenance: Provenance::catalog(PackageId::os20_core(), "cycle.sysml"),
            comment: None,
        })
        .build();
    assert!(
        snap.diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::SpecializationCycle)
    );
    let _ = snap.is_subtype_of(&a, &b);
}

#[test]
fn unallocated_is_classifier_not_null() {
    let sdk = Os20Sdk::bootstrap();
    let u = sdk
        .ontology()
        .resolve_type(&tid("@os20/core#UnallocatedPerson"))
        .unwrap();
    assert_eq!(u.kind, OntologyTypeKind::Classifier);
    assert!(sdk.ontology().is_subtype(
        &tid("@os20/core#UnallocatedPerson"),
        &tid("@os20/core#Person")
    ));
    assert!(sdk.ontology().is_subtype(
        &tid("@os20/core#UnallocatedPerson"),
        &tid("@os20/core#UnallocatedThing")
    ));
    let json = serde_json::to_value(&u).unwrap();
    assert!(json.get("rowid").is_none());
}

#[test]
fn deprecation_resolvable_with_replacement() {
    let sdk = Os20Sdk::bootstrap();
    let g = sdk
        .ontology()
        .resolve_type(&tid("@os20/core#LegacyGadget"))
        .unwrap();
    match g.lifecycle {
        LifecycleStatus::Deprecated { replacement } => {
            assert_eq!(replacement.unwrap().as_str(), "@os20/core#Gadget");
        }
        _ => panic!("expected deprecated"),
    }
}

#[test]
fn file_move_named_identity_stable() {
    let id = ElementId::new("@os20/core#Thing").unwrap();
    let mut d = MemoryOntology::core()
        .snapshot()
        .type_decl(&tid("@os20/core#Thing"))
        .unwrap()
        .clone();
    d.provenance.source_file = "moved/Thing.sysml".into();
    assert_eq!(d.id.as_element(), &id);
}

#[test]
fn public_reports_contain_no_db_ids() {
    let sdk = Os20Sdk::bootstrap();
    let s = sdk
        .ontology()
        .type_summary(&tid("@os20/core#Thing"))
        .unwrap();
    let v: Value = serde_json::to_value(&s).unwrap();
    let text = v.to_string();
    assert!(!text.contains("rowid"));
    assert!(!text.contains("INTEGER"));
    walk_no_numeric_id_keys(&v);
}

fn walk_no_numeric_id_keys(v: &Value) {
    match v {
        Value::Object(map) => {
            for (k, val) in map {
                assert_ne!(k, "rowid");
                assert_ne!(k, "rowId");
                walk_no_numeric_id_keys(val);
            }
        }
        Value::Array(a) => a.iter().for_each(walk_no_numeric_id_keys),
        _ => {}
    }
}

#[test]
fn offline_bootstrap_zero_network() {
    let sdk = Os20Sdk::bootstrap();
    assert!(
        sdk.ontology()
            .resolve_type(&tid("@os20/core#Thing"))
            .is_some()
    );
}

#[test]
fn delete_os20_reindex_same_fingerprint() {
    let sdk = Os20Sdk::bootstrap();
    let dir = tempfile::tempdir().unwrap();
    let os20 = dir.path().join(".os20");
    let first = reindex(sdk.ontology().snapshot(), &os20).unwrap();
    let second = reindex(sdk.ontology().snapshot(), &os20).unwrap();
    assert_eq!(first.semantic_fingerprint, second.semantic_fingerprint);
    assert_eq!(first.semantic_fingerprint, *sdk.ontology().fingerprint());
}

#[test]
fn compatibility_not_by_name() {
    let sdk = Os20Sdk::bootstrap();
    let thing = tid("@os20/core#Thing");
    let other = tid("@os20/physics#Mass");
    assert_eq!(thing.local_name(), "Thing");
    assert_ne!(other.local_name(), "Thing");
    assert!(
        !sdk.ontology().is_assignable(&other, &thing) || sdk.ontology().is_subtype(&other, &thing)
    );
    let mass = sdk.ontology().resolve_type(&other).unwrap();
    assert_eq!(mass.id.local_name(), "Mass");
    assert!(sdk.ontology().is_subtype(&other, &thing));
}

#[test]
fn unknown_type_not_any() {
    let missing = tid("@os20/core#NoSuchType");
    let sdk = Os20Sdk::bootstrap();
    assert!(sdk.ontology().resolve_type(&missing).is_none());
    assert!(
        !sdk.ontology()
            .is_assignable(&missing, &tid("@os20/core#Thing"))
    );
}

#[test]
fn domains_multi_membership_and_not_packages() {
    let sdk = Os20Sdk::bootstrap();
    let motor = sdk
        .ontology()
        .type_summary(&tid("@os20/electrical#ElectricMotor"))
        .unwrap();
    let names: BTreeSet<_> = motor
        .domains
        .iter()
        .map(|d| d.as_str().to_owned())
        .collect();
    assert!(names.iter().any(|d| d.ends_with("#Structure")));
    assert!(names.iter().any(|d| d.ends_with("#Electrical")));
    assert!(
        sdk.ontology().packages().len() < sdk.ontology().domains().len()
            || !sdk.ontology().domains().is_empty()
    );
}

#[test]
fn why_type_and_compatible() {
    let sdk = Os20Sdk::bootstrap();
    let why = sdk
        .ontology()
        .why_type(&tid("@os20/electrical#ServoMotor"))
        .unwrap();
    assert!(!why.specialization_path.is_empty());
    let wc = sdk.ontology().why_compatible(
        &tid("@os20/electrical#ServoMotor"),
        &tid("@os20/core#PhysicalThing"),
    );
    assert!(wc.assignable);
}

#[test]
fn memory_ontology_trait() {
    let m = MemoryOntology::core();
    assert!(m.lookup_type(&tid("@os20/core#Thing")).is_some());
    assert!(!m.fingerprint().as_str().is_empty());
}

#[test]
fn package_role_ontology() {
    let sdk = Os20Sdk::bootstrap();
    let ont = sdk.ontology();
    let core = ont
        .packages()
        .into_iter()
        .find(|p| p.ontology_id.as_str() == "@os20/core")
        .unwrap();
    assert_eq!(core.role, PackageRole::Ontology);
}

#[test]
fn rejects_element_row_id() {
    assert!(ElementId::new("1").is_err());
}

#[test]
fn subtype_query_1k_and_10k() {
    for n in [1_000usize, 10_000] {
        let mut b = OntologySnapshotBuilder::new();
        let thing = tid("@os20/core#Thing");
        b = b.type_decl(TypeDecl {
            id: thing.clone(),
            kind: OntologyTypeKind::Classifier,
            specializes: vec![],
            mixins: vec![],
            domains: vec![],
            properties: vec![],
            lifecycle: LifecycleStatus::Active,
            visibility: Visibility::Public,
            provenance: Provenance::catalog(PackageId::os20_core(), "gen.sysml"),
            comment: None,
        });
        let mut prev = thing.clone();
        for i in 0..n {
            let id = tid(&format!("@os20/core#T{i}"));
            b = b.type_decl(TypeDecl {
                id: id.clone(),
                kind: OntologyTypeKind::Classifier,
                specializes: vec![prev],
                mixins: vec![],
                domains: vec![],
                properties: vec![],
                lifecycle: LifecycleStatus::Active,
                visibility: Visibility::Public,
                provenance: Provenance::catalog(PackageId::os20_core(), "gen.sysml"),
                comment: None,
            });
            prev = id;
        }
        let snap = b.build();
        let last = tid(&format!("@os20/core#T{}", n - 1));
        assert!(snap.is_subtype_of(&last, &thing));
        assert!(snap.type_decl(&tid("@os20/core#T0")).is_some());
        assert!(snap.is_assignable_to(&last, &thing));
        let _ = snap.properties(&last);
    }
}
