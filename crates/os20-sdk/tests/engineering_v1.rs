//! OS20 engineering ontology v1 conformance.

use os20_ontology_core::{
    DiagnosticCode, EngineeringGraph, EngineeringNode, EngineeringPropertyValue, OntologyCompiler,
    Query, TypeId, compile_engineering_v1, electric_powertrain_graph, query, to_jsonld,
    validate_engineering_graph,
};
use os20_sdk::Os20Sdk;

fn snap() -> os20_ontology_core::OntologySnapshot {
    compile_engineering_v1().snapshot
}

#[test]
fn engineering_packages_compile_without_errors() {
    let r = compile_engineering_v1();
    let errors: Vec<_> = r
        .diagnostics
        .iter()
        .filter(|d| d.code.is_error())
        .map(|d| format!("{} {}", d.token, d.message))
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    let sdk = Os20Sdk::from_compile(r);
    assert!(
        sdk.ontology()
            .resolve_type(&TypeId::new("@os20/engineering#EngineeringDefinition").unwrap())
            .is_some()
    );
    assert!(
        sdk.ontology()
            .resolve_type(&TypeId::new("@os20/physical#TractionMotorDefinition").unwrap())
            .is_some()
    );
}

#[test]
fn definition_distinct_from_instance() {
    let s = snap();
    let def = TypeId::new("@os20/engineering#EngineeringDefinition").unwrap();
    let inst = TypeId::new("@os20/engineering#EngineeringInstance").unwrap();
    assert!(!s.is_subtype_of(&def, &inst));
    assert!(!s.is_subtype_of(&inst, &def));
}

#[test]
fn powertrain_graph_validates() {
    let s = snap();
    let g = electric_powertrain_graph();
    let diags = validate_engineering_graph(&s, &g);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.code.is_error())
        .map(|d| format!("{} {}", d.token, d.message))
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    assert!(g.nodes.len() >= 30);
    assert!(g.edges.len() >= 40);
}

#[test]
fn composition_cycle_is_diagnosed() {
    let s = snap();
    let mut g = EngineeringGraph::new();
    g.insert(EngineeringNode::definition(
        "@os20/powertrain#PartDef",
        "@os20/engineering#PartDefinition",
    ));
    g.insert(EngineeringNode::instance(
        "@os20/powertrain#A",
        "@os20/engineering#PartInstance",
        "@os20/engineering#PartDefinition",
    ));
    g.insert(EngineeringNode::instance(
        "@os20/powertrain#B",
        "@os20/engineering#PartInstance",
        "@os20/engineering#PartDefinition",
    ));
    g.edge(
        "@os20/engineering#instanceOf",
        "@os20/powertrain#A",
        "@os20/powertrain#PartDef",
    );
    g.edge(
        "@os20/engineering#instanceOf",
        "@os20/powertrain#B",
        "@os20/powertrain#PartDef",
    );
    g.edge(
        "@os20/core#contains",
        "@os20/powertrain#A",
        "@os20/powertrain#B",
    );
    g.edge(
        "@os20/core#contains",
        "@os20/powertrain#B",
        "@os20/powertrain#A",
    );
    let diags = validate_engineering_graph(&s, &g);
    assert!(
        diags
            .iter()
            .any(|d| d.code == DiagnosticCode::CompositionCycle)
    );
}

#[test]
fn instance_of_must_target_definition() {
    let s = snap();
    let mut g = EngineeringGraph::new();
    g.insert(EngineeringNode::instance(
        "@os20/powertrain#A",
        "@os20/engineering#PartInstance",
        "@os20/engineering#PartInstance",
    ));
    g.insert(EngineeringNode::instance(
        "@os20/powertrain#B",
        "@os20/engineering#PartInstance",
        "@os20/engineering#PartDefinition",
    ));
    g.edge(
        "@os20/engineering#instanceOf",
        "@os20/powertrain#A",
        "@os20/powertrain#B",
    );
    let diags = validate_engineering_graph(&s, &g);
    assert!(
        diags
            .iter()
            .any(|d| d.code == DiagnosticCode::InstanceOfNotDefinition)
    );
}

#[test]
fn valid_quantity_property_and_dimension_mismatch() {
    let s = snap();
    let g = electric_powertrain_graph();
    let diags = validate_engineering_graph(&s, &g);
    assert!(
        !diags
            .iter()
            .any(|d| d.code == DiagnosticCode::QuantityDimensionMismatch)
    );

    let mut bad = EngineeringGraph::new();
    let mut n = EngineeringNode::definition(
        "@os20/powertrain#BadQty",
        "@os20/engineering#QuantityProperty",
    );
    n.properties.insert(
        "value".into(),
        EngineeringPropertyValue::Quantity {
            quantity_kind: "@os20/physical#Torque".into(),
            dimension: "L".into(),
            value: "1".into(),
            unit: "m".into(),
            origin: "calculated".into(),
        },
    );
    bad.insert(n);
    let diags = validate_engineering_graph(&s, &bad);
    assert!(
        diags
            .iter()
            .any(|d| d.code == DiagnosticCode::QuantityDimensionMismatch)
    );
}

#[test]
fn geometry_is_not_the_cad_file() {
    let g = electric_powertrain_graph();
    let geom = g.node("@os20/powertrain#TractionMotorGeometry").unwrap();
    let art = g.node("@os20/powertrain#MotorStepArtifact").unwrap();
    assert_ne!(geom.id, art.id);
    assert_eq!(art.artifact.as_ref().unwrap().format, "step");
    assert!(g.edges.iter().any(|e| {
        e.predicate.as_str() == "@os20/engineering#representedBy"
            && e.source.as_str() == "@os20/powertrain#TractionMotorGeometry"
    }));
}

#[test]
fn port_compatible_and_incompatible() {
    let s = snap();
    let g = electric_powertrain_graph();
    assert!(
        !validate_engineering_graph(&s, &g)
            .iter()
            .any(|d| d.code == DiagnosticCode::PortIncompatible)
    );

    let mut bad = EngineeringGraph::new();
    let mut a = EngineeringNode::instance(
        "@os20/powertrain#P1",
        "@os20/engineering#PortInstance",
        "@os20/engineering#ElectricalPort",
    );
    a.flow_kind = Some("electrical".into());
    a.direction = Some("out".into());
    let mut b = EngineeringNode::instance(
        "@os20/powertrain#P2",
        "@os20/engineering#PortInstance",
        "@os20/engineering#MechanicalPort",
    );
    b.flow_kind = Some("mechanical".into());
    b.direction = Some("in".into());
    bad.insert(a);
    bad.insert(b);
    bad.insert(EngineeringNode::instance(
        "@os20/powertrain#C",
        "@os20/engineering#ConnectionInstance",
        "@os20/engineering#ConnectionDefinition",
    ));
    bad.insert(EngineeringNode::definition(
        "@os20/powertrain#CDef",
        "@os20/engineering#ConnectionDefinition",
    ));
    bad.edge(
        "@os20/engineering#instanceOf",
        "@os20/powertrain#C",
        "@os20/powertrain#CDef",
    );
    bad.edge(
        "@os20/engineering#connects",
        "@os20/powertrain#C",
        "@os20/powertrain#P1",
    );
    bad.edge(
        "@os20/engineering#connects",
        "@os20/powertrain#C",
        "@os20/powertrain#P2",
    );
    let diags = validate_engineering_graph(&s, &bad);
    assert!(
        diags
            .iter()
            .any(|d| d.code == DiagnosticCode::PortIncompatible)
    );
}

#[test]
fn simulation_and_test_triples_stay_distinct() {
    let g = electric_powertrain_graph();
    assert_ne!(
        g.node("@os20/powertrain#ThermalSimDef").unwrap().id,
        g.node("@os20/powertrain#ThermalSimRun").unwrap().id
    );
    assert_ne!(
        g.node("@os20/powertrain#ThermalSimRun").unwrap().id,
        g.node("@os20/powertrain#ThermalSimResult").unwrap().id
    );
    assert_ne!(
        g.node("@os20/powertrain#DynoTestDef").unwrap().id,
        g.node("@os20/powertrain#DynoTestExec").unwrap().id
    );
    let mut missing = g.clone();
    missing.nodes.remove("@os20/powertrain#ThermalSimRun");
    missing.edges.retain(|e| {
        e.source.as_str() != "@os20/powertrain#ThermalSimRun"
            && e.target.as_str() != "@os20/powertrain#ThermalSimRun"
    });
    let s = snap();
    assert!(
        validate_engineering_graph(&s, &missing)
            .iter()
            .any(|d| d.code == DiagnosticCode::SimulationResultMissingRun)
    );
}

#[test]
fn package_export_and_configuration() {
    let s = snap();
    let g = electric_powertrain_graph();
    let a = query(
        &s,
        &g,
        Query::PackageProvides,
        "@os20/powertrain#TractionMotorDef",
    );
    assert!(
        a.ids
            .iter()
            .any(|id| id.contains("Supplier") || id.contains("motor"))
    );
    let mut bad = g.clone();
    bad.edge(
        "@os20/engineering#selects",
        "@os20/powertrain#VehicleConfiguration",
        "@os20/powertrain#EconomyMotorOption",
    );
    assert!(
        validate_engineering_graph(&s, &bad)
            .iter()
            .any(|d| d.code == DiagnosticCode::ConfigurationIncompatible)
    );
}

#[test]
fn unknown_schema_fails_closed() {
    let s = snap();
    let mut g = EngineeringGraph::new();
    g.schema = 99;
    let diags = validate_engineering_graph(&s, &g);
    assert!(
        diags
            .iter()
            .any(|d| d.code == DiagnosticCode::UnknownEngineeringSchema)
    );
}

#[test]
fn dangling_id_and_provenance_chain() {
    let s = snap();
    let mut g = electric_powertrain_graph();
    g.edge(
        "@os20/engineering#derivedFrom",
        "@os20/powertrain#ThermalSimResult",
        "@os20/powertrain#DoesNotExist",
    );
    assert!(
        validate_engineering_graph(&s, &g)
            .iter()
            .any(|d| d.code == DiagnosticCode::DanglingSemanticId)
    );
}

#[test]
fn jsonld_preserves_stable_ids() {
    let s = snap();
    let g = electric_powertrain_graph();
    let doc = to_jsonld(&s, &g);
    let text = serde_json::to_string_pretty(&doc).unwrap();
    assert!(text.contains("\"@context\""));
    assert!(text.contains("@os20/powertrain#TractionMotor"));
    assert!(text.contains("instanceOf") || text.contains("@type"));
    assert!(!text.contains("rowid"));
}

#[test]
fn semantic_queries_on_powertrain() {
    let s = snap();
    let g = electric_powertrain_graph();
    let parts = query(
        &s,
        &g,
        Query::RecursiveParts,
        "@os20/powertrain#EVPowertrain",
    );
    assert!(parts.ids.iter().any(|id| id.ends_with("#TractionMotor")));
    assert!(parts.ids.iter().any(|id| id.ends_with("#MotorHousing")));
    let geom = query(&s, &g, Query::GeometryOf, "@os20/powertrain#TractionMotor");
    assert!(
        geom.ids
            .iter()
            .any(|id| id.ends_with("#TractionMotorGeometry"))
    );
    let req = query(
        &s,
        &g,
        Query::AllocatedRequirements,
        "@os20/powertrain#TractionMotor",
    );
    assert!(req.ids.iter().any(|id| id.ends_with("#TorqueRequirement")));
    let ev = query(
        &s,
        &g,
        Query::EvidenceFor,
        "@os20/powertrain#TorqueRequirement",
    );
    assert!(ev.ids.iter().any(|id| id.ends_with("#DynoTestResult")));
    let tests = query(
        &s,
        &g,
        Query::TestsVerifying,
        "@os20/powertrain#TorqueRequirement",
    );
    assert!(tests.ids.iter().any(|id| id.ends_with("#DynoTestDef")));
    let sims = query(
        &s,
        &g,
        Query::SimulationsUsingGeometry,
        "@os20/powertrain#TractionMotorGeometry",
    );
    assert!(sims.ids.iter().any(|id| id.ends_with("#ThermalSimDef")));
    let inst = query(
        &s,
        &g,
        Query::InstancesOf,
        "@os20/powertrain#TractionMotorDef",
    );
    assert!(inst.ids.iter().any(|id| id.ends_with("#TractionMotor")));
    let unsat = query(&s, &g, Query::UnsatisfiedRequirements, "");
    assert!(unsat.ids.iter().any(|id| id.ends_with("#MassRequirement")));
    let elec = query(
        &s,
        &g,
        Query::ElectricallyConnected,
        "@os20/powertrain#TractionMotor",
    );
    assert!(elec.ids.iter().any(|id| id.ends_with("#MotorController")));
}

#[test]
fn compile_via_sdk_compiler_entry() {
    let r = OntologyCompiler::compile(os20_ontology_core::engineering_compile_request());
    assert!(r.packages.len() >= 6);
}
