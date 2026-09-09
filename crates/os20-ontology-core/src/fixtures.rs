//! Small real ontology package fixtures (`@os20/core`, physics, electrical, …).

use crate::identity::PackageId;
use crate::package::{PackageRole, SourceSpan, Visibility};
use crate::source_graph::{
    BoundElement, BoundFeature, BoundRelation, BoundSourceGraph, OntologyManifest,
};

fn el(
    name: &str,
    kind: &str,
    specs: &[&str],
    file: &str,
    domains: &[&str],
    features: Vec<BoundFeature>,
) -> BoundElement {
    BoundElement {
        qualified: None,
        name: name.into(),
        kind: kind.into(),
        visibility: Visibility::Public,
        specializes: specs.iter().map(|s| (*s).to_string()).collect(),
        mixins: vec![],
        features,
        domains: domains.iter().map(|s| (*s).to_string()).collect(),
        quantity_dimension: None,
        preferred_unit: None,
        value_type: None,
        replaced_by: None,
        split_into: vec![],
        lifecycle: crate::package::LifecycleStatus::Active,
        comment: None,
        inverse: None,
        transitive: false,
        symmetric: false,
        file: file.into(),
        span: SourceSpan { start: 0, end: 32 },
    }
}

fn qty(name: &str, dim: &str, file: &str) -> BoundElement {
    let mut e = el(name, "Quantity", &["Thing"], file, &["Physics"], vec![]);
    e.quantity_dimension = Some(dim.into());
    e.value_type = Some("@omg/kerml#Real".into());
    e.preferred_unit = Some(name.into());
    e
}

fn feat(name: &str, ty: &str) -> BoundFeature {
    BoundFeature {
        name: name.into(),
        ty: ty.into(),
        default_value: None,
    }
}

/// `@omg/kerml` scalars.
pub fn kerml_graph() -> BoundSourceGraph {
    BoundSourceGraph {
        package: "@omg/kerml".into(),
        file_comments: vec![],
        relations: vec![],
        elements: ["Boolean", "Integer", "Real", "String"]
            .into_iter()
            .map(|n| el(n, "DataType", &[], "ScalarValues.kerml", &[], vec![]))
            .collect(),
    }
}

/// `@os20/core` engineering root.
pub fn core_graph() -> BoundSourceGraph {
    BoundSourceGraph {
        package: "@os20/core".into(),
        file_comments: vec![],
        relations: vec![BoundRelation {
            kind: "Specialize".into(),
            source: "PhysicalThing".into(),
            target: "Thing".into(),
        }],
        elements: vec![
            el(
                "Thing",
                "Classifier",
                &[],
                "Thing.sysml",
                &["Design"],
                vec![],
            ),
            el(
                "AbstractThing",
                "Classifier",
                &["Thing"],
                "Thing.sysml",
                &["Design"],
                vec![],
            ),
            el(
                "PhysicalThing",
                "Classifier",
                &["Thing"],
                "PhysicalThing.sysml",
                &["Structure", "Physics"],
                vec![feat("position", "@os20/physics#Length")],
            ),
            el(
                "InformationThing",
                "Classifier",
                &["Thing"],
                "Thing.sysml",
                &["Design"],
                vec![],
            ),
            el(
                "BehaviouralThing",
                "Classifier",
                &["Thing"],
                "Thing.sysml",
                &["Behaviour"],
                vec![],
            ),
            el(
                "UnallocatedThing",
                "Classifier",
                &["Thing"],
                "Unallocated.sysml",
                &["Design"],
                vec![],
            ),
            el(
                "Machine",
                "Classifier",
                &["PhysicalThing"],
                "Machine.sysml",
                &["Structure", "Manufacturing"],
                vec![],
            ),
            el("Design", "Domain", &[], "Domains.sysml", &[], vec![]),
            el("Structure", "Domain", &[], "Domains.sysml", &[], vec![]),
            el("Physics", "Domain", &[], "Domains.sysml", &[], vec![]),
            el("Behaviour", "Domain", &[], "Domains.sysml", &[], vec![]),
            el("Manufacturing", "Domain", &[], "Domains.sysml", &[], vec![]),
            el("Electrical", "Domain", &[], "Domains.sysml", &[], vec![]),
        ],
    }
}

/// Physics quantities.
pub fn physics_graph() -> BoundSourceGraph {
    BoundSourceGraph {
        package: "@os20/physics".into(),
        file_comments: vec![],
        relations: vec![],
        elements: vec![
            qty("Mass", "M", "Quantities.sysml"),
            qty("Length", "L", "Quantities.sysml"),
            qty("Time", "T", "Quantities.sysml"),
            qty("Temperature", "Θ", "Quantities.sysml"),
            qty("Energy", "M L^2 T^-2", "Quantities.sysml"),
            qty("Power", "M L^2 T^-3", "Quantities.sysml"),
        ],
    }
}

/// Electrical ontology (no equations).
pub fn electrical_graph() -> BoundSourceGraph {
    BoundSourceGraph {
        package: "@os20/electrical".into(),
        file_comments: vec![],
        relations: vec![],
        elements: vec![
            qty("Voltage", "M L^2 T^-3 I^-1", "Electrical.sysml"),
            qty("Current", "I", "Electrical.sysml"),
            qty("Resistance", "M L^2 T^-3 I^-2", "Electrical.sysml"),
            el(
                "ElectricMachine",
                "Classifier",
                &["Machine"],
                "Electrical.sysml",
                &["Electrical", "Structure", "Behaviour"],
                vec![feat("ratedVoltage", "Voltage")],
            ),
            el(
                "ElectricMotor",
                "Classifier",
                &["ElectricMachine"],
                "Electrical.sysml",
                &["Electrical", "Structure"],
                vec![],
            ),
            el(
                "ServoMotor",
                "Classifier",
                &["ElectricMotor"],
                "Electrical.sysml",
                &["Electrical", "Structure", "Behaviour"],
                vec![],
            ),
        ],
    }
}

/// Mechanical ontology.
pub fn mechanical_graph() -> BoundSourceGraph {
    BoundSourceGraph {
        package: "@os20/mechanical".into(),
        file_comments: vec![],
        relations: vec![],
        elements: vec![el(
            "RigidBody",
            "Classifier",
            &["PhysicalThing"],
            "Mechanical.sysml",
            &["Structure"],
            vec![feat("mass", "@os20/physics#Mass")],
        )],
    }
}

/// Behaviour ontology.
pub fn behaviour_graph() -> BoundSourceGraph {
    BoundSourceGraph {
        package: "@os20/behaviour".into(),
        file_comments: vec![],
        relations: vec![],
        elements: vec![el(
            "StateMachine",
            "Classifier",
            &["BehaviouralThing"],
            "Behaviour.sysml",
            &["Behaviour"],
            vec![],
        )],
    }
}

/// Physics 2.0 incompatible (Mass no longer in the package).
pub fn physics_v2_graph() -> BoundSourceGraph {
    let mut g = physics_graph();
    g.elements.retain(|e| e.name != "Mass");
    g.elements.push(el(
        "MassV2",
        "Quantity",
        &["Thing"],
        "Quantities.sysml",
        &["Physics"],
        vec![],
    ));
    g
}

fn manifest(name: PackageId, version: &str, deps: Vec<PackageId>, root: &str) -> OntologyManifest {
    OntologyManifest {
        name,
        version: version.into(),
        role: PackageRole::Ontology,
        dependencies: deps,
        optional_dependencies: vec![],
        package_root: root.into(),
    }
}

/// Standard library manifest.
pub fn kerml_manifest() -> OntologyManifest {
    OntologyManifest {
        name: PackageId::omg_kerml(),
        version: "1.0.0".into(),
        role: PackageRole::StandardLibrary,
        dependencies: vec![],
        optional_dependencies: vec![],
        package_root: "kerml".into(),
    }
}

/// Core manifest.
pub fn core_manifest() -> OntologyManifest {
    manifest(
        PackageId::os20_core(),
        "0.1.0",
        vec![PackageId::omg_kerml()],
        "core",
    )
}

/// Physics 0.1.
pub fn physics_manifest() -> OntologyManifest {
    manifest(
        PackageId::os20_physics(),
        "0.1.0",
        vec![PackageId::os20_core()],
        "physics",
    )
}

/// Physics 2.0.
pub fn physics_v2_manifest() -> OntologyManifest {
    manifest(
        PackageId::os20_physics(),
        "2.0.0",
        vec![PackageId::os20_core()],
        "physics",
    )
}

/// Electrical.
pub fn electrical_manifest() -> OntologyManifest {
    manifest(
        PackageId::os20_electrical(),
        "0.1.0",
        vec![PackageId::os20_core(), PackageId::os20_physics()],
        "electrical",
    )
}

/// Mechanical.
pub fn mechanical_manifest() -> OntologyManifest {
    manifest(
        PackageId::os20_mechanical(),
        "0.1.0",
        vec![PackageId::os20_core(), PackageId::os20_physics()],
        "mechanical",
    )
}

/// Behaviour.
pub fn behaviour_manifest() -> OntologyManifest {
    manifest(
        PackageId::os20_behaviour(),
        "0.1.0",
        vec![PackageId::os20_core()],
        "behaviour",
    )
}

/// SysML/KerML source for Git authority (not parsed by the compiler).
pub fn authored_source(package: &str) -> &'static str {
    match package {
        "@os20/core" => {
            "package Core {\n  part def Thing;\n  part def PhysicalThing :> Thing;\n  part def Machine :> PhysicalThing;\n}\n"
        }
        "@os20/physics" => "package Physics { /* Mass Length Time Temperature Energy Power */ }\n",
        "@os20/electrical" => {
            "package Electrical {\n  part def ElectricMachine :> Machine;\n  attribute voltage;\n}\n"
        }
        "@os20/mechanical" => "package Mechanical { part def RigidBody :> PhysicalThing; }\n",
        "@os20/behaviour" => "package Behaviour { part def StateMachine :> BehaviouralThing; }\n",
        "@omg/kerml" => {
            "package ScalarValues { datatype Boolean; datatype Integer; datatype Real; datatype String; }\n"
        }
        _ => "",
    }
}
