//! Semantic property IDs: owner-qualified PropertyId + canonical os20:property bindings.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use os20_sdk::{
    DiagnosticCode, Os20Sdk, PackageId, PropertyId, SemanticPropertyId, SemanticPropertyRegistry,
    TypeId,
};

fn tid(s: &str) -> TypeId {
    TypeId::new(s).unwrap()
}

fn catalog_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/catalog")
}

fn load_catalog_registry() -> SemanticPropertyRegistry {
    let dir = catalog_dir();
    let mut r = SemanticPropertyRegistry::new();
    r.load_properties_json(
        &fs::read_to_string(dir.join("semantic-properties.json")).expect("properties json"),
    )
    .expect("load properties");
    r.load_bindings_json(
        &fs::read_to_string(dir.join("semantic-property-bindings.json")).expect("bindings json"),
    )
    .expect("load bindings");
    assert!(
        r.diagnostics().is_empty(),
        "catalog sidecars must load cleanly: {:?}",
        r.diagnostics()
    );
    r
}

#[test]
fn two_types_same_local_mass_get_distinct_property_ids() {
    let pkg = PackageId::new("@example/demo").unwrap();
    let a = PropertyId::on_owner(&pkg, "Compressor", "mass");
    let b = PropertyId::on_owner(&pkg, "Fan", "mass");
    assert_ne!(a, b);
    assert_eq!(a.as_str(), "@example/demo#Compressor.mass");
    assert_eq!(b.as_str(), "@example/demo#Fan.mass");
}

#[test]
fn bootstrap_mass_is_owner_qualified_not_bare() {
    let sdk = Os20Sdk::bootstrap();
    let props = sdk.ontology().properties(&tid("@os20/electrical#ServoMotor"));
    assert!(
        props
            .iter()
            .any(|p| p.id.as_str() == "@os20/core#MassBearing.mass")
    );
    assert!(!props.iter().any(|p| p.id.as_str() == "@os20/core#mass"));
}

#[test]
fn two_attributes_share_one_canonical_semantic_meaning() {
    let mut r = load_catalog_registry();
    r.load_bindings_json(
        r#"{
          "bindings": [
            { "sysmlFeature": "OS20_HVAC::Compressor::mass", "semanticId": "os20:property:equipment.dryMass" },
            { "sysmlFeature": "OS20_HVAC::Fan::mass", "semanticId": "os20:property:equipment.dryMass" }
          ]
        }"#,
    )
    .unwrap();
    let a = r.resolve_feature("OS20_HVAC::Compressor::mass").unwrap();
    let b = r.resolve_feature("OS20_HVAC::Fan.mass").unwrap();
    assert_eq!(a.definition.id.as_str(), "os20:property:equipment.dryMass");
    assert_eq!(a.definition.id, b.definition.id);
}

#[test]
fn same_name_different_semantic_meanings() {
    let mut r = load_catalog_registry();
    r.load_bindings_json(
        r#"{
          "bindings": [
            { "sysmlFeature": "OS20_HVAC::Compressor::mass", "semanticId": "os20:property:equipment.dryMass" },
            { "sysmlFeature": "Logistics::Crate::mass", "semanticId": "os20:property:shipment.grossMass" }
          ]
        }"#,
    )
    .unwrap();
    let dry = r.resolve_feature("OS20_HVAC::Compressor::mass").unwrap();
    let gross = r.resolve_feature("Logistics::Crate::mass").unwrap();
    assert_ne!(dry.definition.id, gross.definition.id);
    assert_eq!(dry.definition.name, gross.definition.name);
}

#[test]
fn compatible_quantity_kinds_accepted() {
    let r = load_catalog_registry();
    let id = SemanticPropertyId::new("os20:property:equipment.dryMass").unwrap();
    assert!(r.check_quantity(&id, "mass").is_ok());
    assert!(r.check_quantity(&id, "Mass").is_ok());
}

#[test]
fn incompatible_dimensions_rejected() {
    let r = load_catalog_registry();
    let id = SemanticPropertyId::new("os20:property:equipment.dryMass").unwrap();
    let err = r.check_quantity(&id, "length").unwrap_err();
    assert_eq!(err.code, DiagnosticCode::SemanticQuantityMismatch);
    let flow = SemanticPropertyId::new("os20:property:fluid.massFlowRate").unwrap();
    let err2 = r.check_quantity(&flow, "mass").unwrap_err();
    assert_eq!(err2.code, DiagnosticCode::SemanticQuantityMismatch);
}

#[test]
fn ambiguous_unqualified_reference_diagnostics() {
    let mut r = load_catalog_registry();
    r.load_bindings_json(
        r#"{
          "bindings": [
            { "sysmlFeature": "OS20_HVAC::Compressor::mass", "semanticId": "os20:property:equipment.dryMass" },
            { "sysmlFeature": "Logistics::Crate::mass", "semanticId": "os20:property:shipment.grossMass" }
          ]
        }"#,
    )
    .unwrap();
    let candidates = vec![
        "OS20_HVAC::Compressor::mass".into(),
        "Logistics::Crate::mass".into(),
    ];
    let err = r.resolve_by_local_name("mass", &candidates).unwrap_err();
    assert_eq!(err.code, DiagnosticCode::AmbiguousPropertyReference);
}

#[test]
fn api_returns_correct_semantic_identity() {
    let r = load_catalog_registry();
    let disp = r
        .display_for_feature("@os20/core#MassBearing.mass")
        .expect("bound MassBearing.mass");
    assert_eq!(disp.display_name, "Mass");
    assert_eq!(disp.semantic_meaning, "Equipment dry mass");
    assert_eq!(disp.quantity, "mass");
    assert_eq!(disp.unit.as_deref(), Some("Kilogram"));
    assert_eq!(disp.semantic_id, "os20:property:equipment.dryMass");
    assert_eq!(disp.source_package, "@os20/core");
}

#[test]
fn conflicting_semantic_defs_emit_diagnostic() {
    let mut r = SemanticPropertyRegistry::new();
    r.load_properties_json(
        r#"{
          "properties": [
            {
              "id": "os20:property:equipment.dryMass",
              "name": "Mass",
              "description": "Equipment dry mass",
              "quantityKind": "mass",
              "canonicalUnit": "Kilogram",
              "sourcePackage": "@os20/core",
              "sourceVersion": "0.1.0"
            },
            {
              "id": "os20:property:equipment.dryMass",
              "name": "Mass",
              "description": "Equipment dry mass",
              "quantityKind": "length",
              "canonicalUnit": "Metre",
              "sourcePackage": "@os20/core",
              "sourceVersion": "0.1.0"
            }
          ]
        }"#,
    )
    .unwrap();
    assert!(
        r.diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::ConflictingSemanticProperty)
    );
}

#[test]
fn duplicate_owned_property_code_is_e4027() {
    assert_eq!(DiagnosticCode::DuplicateOwnedProperty.token(), "OS20-E4027");
    assert_eq!(DiagnosticCode::AmbiguousPropertyReference.token(), "OS20-E4028");
    assert_eq!(DiagnosticCode::ConflictingSemanticProperty.token(), "OS20-E4029");
    assert_eq!(DiagnosticCode::SemanticQuantityMismatch.token(), "OS20-E4030");
    assert_eq!(DiagnosticCode::AmbiguousLegacyPropertyName.token(), "OS20-W4002");
    assert!(!DiagnosticCode::AmbiguousLegacyPropertyName.is_error());
}

#[test]
fn never_use_bare_mass_as_global_identity() {
    let mut by_name: BTreeMap<&str, Vec<PropertyId>> = BTreeMap::new();
    let pkg = PackageId::os20_core();
    for owner in ["Compressor", "Fan", "MassBearing"] {
        by_name
            .entry("mass")
            .or_default()
            .push(PropertyId::on_owner(&pkg, owner, "mass"));
    }
    let ids = &by_name["mass"];
    assert_eq!(ids.len(), 3);
    assert_eq!(ids.iter().collect::<BTreeSet<_>>().len(), 3);
}

#[test]
fn existing_packages_continue_working() {
    let sdk = Os20Sdk::bootstrap();
    assert!(sdk.ontology().resolve_type(&tid("@os20/core#Thing")).is_some());
    assert!(sdk.ontology().resolve_type(&tid("@os20/core#PhysicalThing")).is_some());
    let props = sdk.ontology().properties(&tid("@os20/electrical#ServoMotor"));
    assert!(props.iter().any(|p| p.id.as_str().ends_with(".mass")));
    assert!(props.iter().any(|p| p.id.as_str().ends_with(".position")));
}
