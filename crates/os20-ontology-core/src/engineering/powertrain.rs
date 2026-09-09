//! Electric powertrain engineering graph fixture (~40 nodes).

use crate::engineering::graph::{
    ArtifactRef, EngineeringGraph, EngineeringNode, EngineeringPropertyValue,
};
use crate::engineering::relations::CONTAINS;

const P: &str = "@os20/powertrain";

fn n(local: &str) -> String {
    format!("{P}#{local}")
}

fn qty(kind: &str, dim: &str, value: &str, unit: &str, origin: &str) -> EngineeringPropertyValue {
    EngineeringPropertyValue::Quantity {
        quantity_kind: kind.into(),
        dimension: dim.into(),
        value: value.into(),
        unit: unit.into(),
        origin: origin.into(),
    }
}

/// Realistic EV powertrain graph used by conformance tests and JSON-LD export.
pub fn electric_powertrain_graph() -> EngineeringGraph {
    let mut g = EngineeringGraph::new();

    // --- exported definitions (package != system) ---
    let mut motor_def = EngineeringNode::definition(
        &n("TractionMotorDef"),
        "@os20/physical#TractionMotorDefinition",
    );
    motor_def.providing_package = Some("@supplier/motor".into());
    g.insert(motor_def);
    g.insert(EngineeringNode::definition(
        &n("BatteryPackDef"),
        "@os20/physical#Battery",
    ));
    g.insert(EngineeringNode::definition(
        &n("ControllerDef"),
        "@os20/physical#ElectricalComponent",
    ));
    g.insert(EngineeringNode::definition(
        &n("GearboxDef"),
        "@os20/engineering#AssemblyDefinition",
    ));
    g.insert(EngineeringNode::definition(
        &n("CoolingDef"),
        "@os20/engineering#AssemblyDefinition",
    ));
    g.insert(EngineeringNode::definition(
        &n("FirmwareDef"),
        "@os20/software#Firmware",
    ));
    g.insert(EngineeringNode::definition(
        &n("TorqueReqDef"),
        "@os20/assurance#PerformanceRequirement",
    ));
    g.insert(EngineeringNode::definition(
        &n("ThermalReqDef"),
        "@os20/assurance#EnvironmentalRequirement",
    ));
    g.insert(EngineeringNode::definition(
        &n("MassReqDef"),
        "@os20/assurance#ConstraintRequirement",
    ));
    g.insert(EngineeringNode::definition(
        &n("DynoTestDef"),
        "@os20/assurance#TestDefinition",
    ));
    g.insert(EngineeringNode::definition(
        &n("ThermalSimDef"),
        "@os20/analysis#ThermalSimulation",
    ));
    g.insert(EngineeringNode::definition(
        &n("MotorMfgPlan"),
        "@os20/lifecycle#ManufacturingPlan",
    ));

    // --- system / assemblies ---
    g.insert(EngineeringNode::instance(
        &n("EVPowertrain"),
        "@os20/engineering#SystemInstance",
        "@os20/engineering#SystemDefinition",
    ));
    g.insert(EngineeringNode::instance(
        &n("BatteryPack"),
        "@os20/engineering#DeviceInstance",
        "@os20/physical#Battery",
    ));
    g.insert(EngineeringNode::instance(
        &n("MotorController"),
        "@os20/engineering#DeviceInstance",
        "@os20/physical#ElectricalComponent",
    ));
    g.insert(EngineeringNode::instance(
        &n("TractionMotor"),
        "@os20/engineering#DeviceInstance",
        "@os20/physical#TractionMotorDefinition",
    ));
    g.insert(EngineeringNode::instance(
        &n("Gearbox"),
        "@os20/engineering#AssemblyInstance",
        "@os20/engineering#AssemblyDefinition",
    ));
    g.insert(EngineeringNode::instance(
        &n("CoolingSystem"),
        "@os20/engineering#AssemblyInstance",
        "@os20/engineering#AssemblyDefinition",
    ));
    g.insert(EngineeringNode::instance(
        &n("MotorHousing"),
        "@os20/engineering#PartInstance",
        "@os20/engineering#PartDefinition",
    ));

    // --- geometry / materials ---
    g.insert(EngineeringNode::definition(
        &n("TractionMotorGeometry"),
        "@os20/physical#SolidGeometry",
    ));
    g.insert(EngineeringNode::definition(
        &n("CoolingHole"),
        "@os20/physical#HoleFeature",
    ));
    g.insert(EngineeringNode::definition(
        &n("ElectricalSteel"),
        "@os20/physical#Metal",
    ));
    g.insert(EngineeringNode::definition(
        &n("Copper"),
        "@os20/physical#Metal",
    ));
    let mut assign = EngineeringNode::definition(
        &n("StatorCopperAssign"),
        "@os20/physical#MaterialAssignment",
    );
    assign.properties.insert(
        "applies".into(),
        EngineeringPropertyValue::Text {
            value: "stator windings".into(),
        },
    );
    g.insert(assign);

    // --- behavior / software ---
    g.insert(EngineeringNode::definition(
        &n("MotorBehavior"),
        "@os20/engineering#StateMachine",
    ));
    g.insert(EngineeringNode::definition(
        &n("MotorFirmware"),
        "@os20/software#Firmware",
    ));
    let mut scxml =
        EngineeringNode::definition(&n("MotorScxmlArtifact"), "@os20/lifecycle#BehaviorArtifact");
    scxml.artifact = Some(ArtifactRef {
        digest: "artifact:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .into(),
        format: "scxml".into(),
        media_type: Some("application/xml".into()),
    });
    g.insert(scxml);

    // --- ports / connections ---
    insert_port(&mut g, "ShaftPort", "mechanical", "out");
    insert_port(&mut g, "GearboxInputPort", "mechanical", "in");
    insert_port(&mut g, "HVPort", "electrical", "in");
    insert_port(&mut g, "ControllerMotorPort", "electrical", "out");
    insert_port(&mut g, "ControllerBatteryPort", "electrical", "in");
    insert_port(&mut g, "BatteryHVPort", "electrical", "out");
    insert_port(&mut g, "CoolantInPort", "thermal", "in");
    insert_port(&mut g, "CoolantOutPort", "thermal", "out");
    insert_port(&mut g, "CanPort", "data", "inout");
    insert_port(&mut g, "ControllerCanPort", "data", "inout");
    g.insert(EngineeringNode::instance(
        &n("ShaftCoupling"),
        "@os20/engineering#ConnectionInstance",
        "@os20/engineering#ConnectionDefinition",
    ));
    g.insert(EngineeringNode::instance(
        &n("HVBus"),
        "@os20/engineering#ConnectionInstance",
        "@os20/engineering#ConnectionDefinition",
    ));
    g.insert(EngineeringNode::instance(
        &n("DcLink"),
        "@os20/engineering#ConnectionInstance",
        "@os20/engineering#ConnectionDefinition",
    ));
    g.insert(EngineeringNode::instance(
        &n("CoolantLoop"),
        "@os20/engineering#ConnectionInstance",
        "@os20/engineering#ConnectionDefinition",
    ));
    g.insert(EngineeringNode::instance(
        &n("CanBus"),
        "@os20/engineering#ConnectionInstance",
        "@os20/engineering#ConnectionDefinition",
    ));

    // --- properties / constraints ---
    let mut torque =
        EngineeringNode::definition(&n("RatedTorque"), "@os20/engineering#QuantityProperty");
    torque.properties.insert(
        "value".into(),
        qty(
            "@os20/physical#Torque",
            "M L^2 T^-2",
            "250",
            "N.m",
            "calculated",
        ),
    );
    g.insert(torque);
    let mut torque_req = EngineeringNode::definition(
        &n("TorqueRequirement"),
        "@os20/assurance#PerformanceRequirement",
    );
    torque_req.properties.insert(
        "statement".into(),
        EngineeringPropertyValue::Text {
            value: "Motor shall provide at least 20 Nm".into(),
        },
    );
    torque_req.properties.insert(
        "operator".into(),
        EngineeringPropertyValue::Enum {
            value: "gte".into(),
        },
    );
    torque_req.properties.insert(
        "threshold".into(),
        qty(
            "@os20/physical#Torque",
            "M L^2 T^-2",
            "20",
            "N.m",
            "calculated",
        ),
    );
    g.insert(torque_req);
    g.insert(EngineeringNode::definition(
        &n("ThermalRequirement"),
        "@os20/assurance#EnvironmentalRequirement",
    ));
    g.insert(EngineeringNode::definition(
        &n("MassRequirement"),
        "@os20/assurance#ConstraintRequirement",
    ));
    g.insert(EngineeringNode::definition(
        &n("TorqueConstraint"),
        "@os20/assurance#RangeConstraint",
    ));

    // --- simulation / execution / results ---
    let mut run = EngineeringNode::instance(
        &n("ThermalSimRun"),
        "@os20/analysis#SimulationRun",
        "@os20/analysis#ThermalSimulation",
    );
    run.execution_run =
        Some("run:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into());
    g.insert(run);
    let mut sim_res =
        EngineeringNode::definition(&n("ThermalSimResult"), "@os20/analysis#SimulationResult");
    sim_res.is_instance = false;
    let mut field = EngineeringNode::definition(
        &n("TemperatureFieldArtifact"),
        "@os20/lifecycle#SimulationArtifact",
    );
    field.artifact = Some(ArtifactRef {
        digest: "artifact:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            .into(),
        format: "hdf5".into(),
        media_type: None,
    });
    g.insert(field);
    g.insert(sim_res);
    g.insert(EngineeringNode::definition(
        &n("ConjugateHeatSolver"),
        "@os20/analysis#SolverReference",
    ));
    g.insert(EngineeringNode::definition(
        &n("CoolantInletBc"),
        "@os20/analysis#BoundaryCondition",
    ));
    g.insert(EngineeringNode::definition(
        &n("MotorThermalModel"),
        "@os20/analysis#SimulationModel",
    ));

    // --- test ---
    g.insert(EngineeringNode::instance(
        &n("DynoTestExec"),
        "@os20/assurance#TestExecution",
        "@os20/assurance#TestDefinition",
    ));
    g.insert(EngineeringNode::definition(
        &n("DynoTestResult"),
        "@os20/assurance#TestResult",
    ));

    // --- manufacturing ---
    g.insert(EngineeringNode::definition(
        &n("DrillOp"),
        "@os20/lifecycle#ManufacturingOperation",
    ));
    let mut step =
        EngineeringNode::definition(&n("MotorStepArtifact"), "@os20/lifecycle#GeometryArtifact");
    step.artifact = Some(ArtifactRef {
        digest: "artifact:sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
            .into(),
        format: "step".into(),
        media_type: Some("model/step".into()),
    });
    g.insert(step);

    // --- configuration ---
    g.insert(EngineeringNode::definition(
        &n("PerformanceMotorOption"),
        "@os20/engineering#Option",
    ));
    g.insert(EngineeringNode::definition(
        &n("EconomyMotorOption"),
        "@os20/engineering#Option",
    ));
    g.insert(EngineeringNode::instance(
        &n("VehicleConfiguration"),
        "@os20/engineering#Configuration",
        "@os20/engineering#ConfigurationDefinition",
    ));

    // --- evidence wrapper ---
    // TestResult already specializes EngineeringEvidence.

    // edges
    let io = "@os20/engineering#instanceOf";
    g.edge(io, &n("EVPowertrain"), "@os20/engineering#SystemDefinition");
    // EVPowertrain instanceOf SystemDefinition type — need a definition *node*.
    // Replace: add SystemDef node.
    g.insert(EngineeringNode::definition(
        &n("PowertrainDef"),
        "@os20/engineering#SystemDefinition",
    ));
    g.edges
        .retain(|e| !(e.source.as_str() == n("EVPowertrain") && e.predicate.as_str() == io));
    g.edge(io, &n("EVPowertrain"), &n("PowertrainDef"));
    g.edge(io, &n("BatteryPack"), &n("BatteryPackDef"));
    g.edge(io, &n("MotorController"), &n("ControllerDef"));
    g.edge(io, &n("TractionMotor"), &n("TractionMotorDef"));
    g.edge(io, &n("Gearbox"), &n("GearboxDef"));
    g.edge(io, &n("CoolingSystem"), &n("CoolingDef"));
    g.edge(io, &n("MotorHousing"), "@os20/engineering#PartDefinition");

    // Housing instanceOf needs a definition node
    g.insert(EngineeringNode::definition(
        &n("HousingDef"),
        "@os20/engineering#PartDefinition",
    ));
    g.edges
        .retain(|e| !(e.source.as_str() == n("MotorHousing") && e.predicate.as_str() == io));
    g.edge(io, &n("MotorHousing"), &n("HousingDef"));
    g.edge(io, &n("ThermalSimRun"), &n("ThermalSimDef"));
    g.edge(io, &n("DynoTestExec"), &n("DynoTestDef"));
    g.edge(
        io,
        &n("VehicleConfiguration"),
        "@os20/engineering#ConfigurationDefinition",
    );
    g.insert(EngineeringNode::definition(
        &n("ConfigDef"),
        "@os20/engineering#ConfigurationDefinition",
    ));
    g.edges.retain(|e| {
        !(e.source.as_str() == n("VehicleConfiguration") && e.predicate.as_str() == io)
    });
    g.edge(io, &n("VehicleConfiguration"), &n("ConfigDef"));

    for child in [
        "BatteryPack",
        "MotorController",
        "TractionMotor",
        "Gearbox",
        "CoolingSystem",
    ] {
        g.edge(CONTAINS, &n("EVPowertrain"), &n(child));
    }
    g.edge(CONTAINS, &n("TractionMotor"), &n("MotorHousing"));
    g.edge(CONTAINS, &n("TractionMotorGeometry"), &n("CoolingHole"));

    g.edge(
        "@os20/engineering#hasGeometry",
        &n("TractionMotor"),
        &n("TractionMotorGeometry"),
    );
    g.edge(
        "@os20/engineering#representedBy",
        &n("TractionMotorGeometry"),
        &n("MotorStepArtifact"),
    );
    g.edge(
        "@os20/engineering#madeOf",
        &n("MotorHousing"),
        &n("ElectricalSteel"),
    );
    g.edge(
        "@os20/engineering#hasMaterialAssignment",
        &n("TractionMotor"),
        &n("StatorCopperAssign"),
    );
    g.edge(
        "@os20/engineering#appliesToRegion",
        &n("StatorCopperAssign"),
        &n("TractionMotorGeometry"),
    );
    g.edge(
        "@os20/engineering#hasBehavior",
        &n("TractionMotor"),
        &n("MotorBehavior"),
    );
    g.edge(
        "@os20/engineering#representedBy",
        &n("MotorBehavior"),
        &n("MotorScxmlArtifact"),
    );
    g.edge(
        "@os20/engineering#hasProperty",
        &n("TractionMotor"),
        &n("RatedTorque"),
    );
    g.edge(
        "@os20/engineering#allocatedTo",
        &n("TorqueRequirement"),
        &n("TractionMotor"),
    );
    g.edge(
        "@os20/engineering#allocatedTo",
        &n("ThermalRequirement"),
        &n("TractionMotor"),
    );
    g.edge(
        "@os20/engineering#verifiedBy",
        &n("TorqueRequirement"),
        &n("DynoTestDef"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("TractionMotor"),
        &n("ShaftPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("TractionMotor"),
        &n("HVPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("TractionMotor"),
        &n("CoolantInPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("TractionMotor"),
        &n("CoolantOutPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("TractionMotor"),
        &n("CanPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("Gearbox"),
        &n("GearboxInputPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("MotorController"),
        &n("ControllerMotorPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("MotorController"),
        &n("ControllerBatteryPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("MotorController"),
        &n("ControllerCanPort"),
    );
    g.edge(
        "@os20/engineering#hasPort",
        &n("BatteryPack"),
        &n("BatteryHVPort"),
    );
    g.edge(
        "@os20/engineering#connects",
        &n("ShaftCoupling"),
        &n("ShaftPort"),
    );
    g.edge(
        "@os20/engineering#connects",
        &n("ShaftCoupling"),
        &n("GearboxInputPort"),
    );
    g.edge("@os20/engineering#connects", &n("HVBus"), &n("HVPort"));
    g.edge(
        "@os20/engineering#connects",
        &n("HVBus"),
        &n("ControllerMotorPort"),
    );
    g.edge(
        "@os20/engineering#connects",
        &n("DcLink"),
        &n("ControllerBatteryPort"),
    );
    g.edge(
        "@os20/engineering#connects",
        &n("DcLink"),
        &n("BatteryHVPort"),
    );
    g.edge(
        "@os20/engineering#connects",
        &n("CoolantLoop"),
        &n("CoolantInPort"),
    );
    g.edge(
        "@os20/engineering#connects",
        &n("CoolantLoop"),
        &n("CoolantOutPort"),
    );
    g.edge("@os20/engineering#connects", &n("CanBus"), &n("CanPort"));
    g.edge(
        "@os20/engineering#connects",
        &n("CanBus"),
        &n("ControllerCanPort"),
    );
    g.edge(
        "@os20/engineering#controls",
        &n("MotorController"),
        &n("TractionMotor"),
    );
    g.edge(
        "@os20/engineering#executesOn",
        &n("MotorFirmware"),
        &n("MotorController"),
    );
    g.edge(
        "@os20/engineering#simulates",
        &n("ThermalSimDef"),
        &n("TractionMotor"),
    );
    g.edge(
        "@os20/engineering#usesGeometry",
        &n("ThermalSimDef"),
        &n("TractionMotorGeometry"),
    );
    g.edge(
        "@os20/engineering#usesMaterial",
        &n("ThermalSimDef"),
        &n("Copper"),
    );
    g.edge(
        "@os20/engineering#usesBoundaryCondition",
        &n("ThermalSimDef"),
        &n("CoolantInletBc"),
    );
    g.edge(
        "@os20/engineering#usesSolver",
        &n("ThermalSimDef"),
        &n("ConjugateHeatSolver"),
    );
    g.edge(
        "@os20/engineering#usesModel",
        &n("ThermalSimDef"),
        &n("MotorThermalModel"),
    );
    g.edge(
        "@os20/engineering#producesResult",
        &n("ThermalSimRun"),
        &n("ThermalSimResult"),
    );
    g.edge(
        "@os20/engineering#generatedBy",
        &n("ThermalSimResult"),
        &n("ThermalSimRun"),
    );
    g.edge(
        "@os20/engineering#derivedFrom",
        &n("ThermalSimResult"),
        &n("ThermalSimRun"),
    );
    g.edge(
        "@os20/engineering#representedBy",
        &n("ThermalSimResult"),
        &n("TemperatureFieldArtifact"),
    );
    g.edge(
        "@os20/engineering#producesResult",
        &n("DynoTestExec"),
        &n("DynoTestResult"),
    );
    g.edge(
        "@os20/engineering#generatedBy",
        &n("DynoTestResult"),
        &n("DynoTestExec"),
    );
    g.edge(
        "@os20/engineering#providesEvidenceFor",
        &n("DynoTestResult"),
        &n("TorqueRequirement"),
    );
    g.edge(
        "@os20/engineering#providesEvidenceFor",
        &n("ThermalSimResult"),
        &n("ThermalRequirement"),
    );
    g.edge(
        "@os20/engineering#manufacturedBy",
        &n("TractionMotor"),
        &n("MotorMfgPlan"),
    );
    g.edge(
        "@os20/engineering#producedBy",
        &n("CoolingHole"),
        &n("DrillOp"),
    );
    g.edge(
        "@os20/engineering#configures",
        &n("VehicleConfiguration"),
        &n("EVPowertrain"),
    );
    g.edge(
        "@os20/engineering#selects",
        &n("VehicleConfiguration"),
        &n("PerformanceMotorOption"),
    );
    g.edge(
        "@os20/engineering#excludes",
        &n("PerformanceMotorOption"),
        &n("EconomyMotorOption"),
    );
    g.insert(EngineeringNode::definition(
        &n("SupplierMotorPackage"),
        "@os20/engineering#EngineeringThing",
    ));
    g.edge(
        "@os20/engineering#providedBy",
        &n("TractionMotorDef"),
        &n("SupplierMotorPackage"),
    );

    g
}

fn insert_port(g: &mut EngineeringGraph, local: &str, kind: &str, dir: &str) {
    let ty = match kind {
        "mechanical" => "@os20/engineering#PortInstance",
        "electrical" => "@os20/engineering#PortInstance",
        "thermal" => "@os20/engineering#PortInstance",
        "data" => "@os20/engineering#PortInstance",
        _ => "@os20/engineering#PortInstance",
    };
    let def = match kind {
        "mechanical" => "@os20/engineering#MechanicalPort",
        "electrical" => "@os20/engineering#ElectricalPort",
        "thermal" => "@os20/engineering#ThermalPort",
        "data" => "@os20/engineering#DataPort",
        _ => "@os20/engineering#PortDefinition",
    };
    let mut p = EngineeringNode::instance(&n(local), ty, def);
    p.flow_kind = Some(kind.into());
    p.direction = Some(dir.into());
    g.insert(p);
}
