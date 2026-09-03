//! Foundational OS20 core ontology catalog and `MemoryOntology::core()` bootstrap.

use crate::evolution::{EvolutionEdge, EvolutionKind, RefactoringMap, UnallocatedPattern};
use crate::identity::{
    ElementId, PackageId, PredicateId, PropertyId, SemanticDomainId, TypeId, UnitId,
};
use crate::model::{
    OntologyTypeKind, Predicate, PropertyDecl, QuantityType, SemanticDomain, TypeDecl,
};
use crate::package::{
    LifecycleStatus, OntologySourceKind, PackageReleaseRef, PackageRole, Provenance, TypeRef,
    Visibility,
};
use crate::relations::{Multiplicity, RelationAlgebra, RelationType};
use crate::snapshot::{OntologyPackage, OntologySnapshot, OntologySnapshotBuilder};

/// In-memory bootstrap ontology. Do not remove; migrate toward package-backed
/// snapshots from locked `PackageRole::Ontology` pins.
#[derive(Clone, Debug)]
pub struct MemoryOntology {
    snapshot: OntologySnapshot,
}

impl MemoryOntology {
    /// Canonical core bootstrap used by the existing SDK.
    pub fn core() -> Self {
        Self {
            snapshot: build_core_snapshot(None),
        }
    }

    /// Same semantic catalog with an extra comment on `Thing` (source fingerprint only).
    pub fn core_with_thing_comment(comment: &str) -> Self {
        Self {
            snapshot: build_core_snapshot(Some(comment)),
        }
    }

    /// Compiled-in core catalog version (`0.1.0`). Not a process-global mutable core.
    pub fn core_ontology_version() -> &'static str {
        crate::versions::OS20_CORE_ONTOLOGY_VERSION
    }

    /// Snapshot.
    pub fn snapshot(&self) -> &OntologySnapshot {
        &self.snapshot
    }

    /// Into snapshot.
    pub fn into_snapshot(self) -> OntologySnapshot {
        self.snapshot
    }
}

/// Baseline semantic domains (extensible: not a closed enum).
pub fn baseline_domains() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Design", "Design"),
        ("Structure", "Structure"),
        ("Behaviour", "Behaviour"),
        ("Requirements", "Requirements"),
        ("Verification", "Verification"),
        ("Validation", "Validation"),
        ("Simulation", "Simulation"),
        ("Geometry", "Geometry"),
        ("Manufacturing", "Manufacturing"),
        ("Physics", "Physics"),
        ("Safety", "Safety"),
        ("Compliance", "Compliance"),
        ("Electrical", "Electrical"),
    ]
}

fn domain_id(local: &str) -> SemanticDomainId {
    SemanticDomainId::catalog(&PackageId::os20_core(), local)
}

fn t_core(local: &str) -> TypeId {
    TypeId::from_element(ElementId::catalog(&PackageId::os20_core(), local))
}

fn t_phys(local: &str) -> TypeId {
    TypeId::from_element(ElementId::catalog(&PackageId::os20_physics(), local))
}

fn t_el(local: &str) -> TypeId {
    TypeId::from_element(ElementId::catalog(&PackageId::os20_electrical(), local))
}

fn t_kerml(local: &str) -> TypeId {
    TypeId::from_element(ElementId::catalog(&PackageId::omg_kerml(), local))
}

#[allow(clippy::too_many_arguments)]
fn type_decl(
    id: TypeId,
    kind: OntologyTypeKind,
    specs: &[&TypeId],
    mixins: &[&TypeId],
    domains: &[&str],
    properties: Vec<PropertyDecl>,
    file: &str,
    comment: Option<&str>,
    lifecycle: LifecycleStatus,
) -> TypeDecl {
    let package = id.package().expect("type package");
    TypeDecl {
        kind,
        specializes: specs.iter().map(|t| (*t).clone()).collect(),
        mixins: mixins.iter().map(|t| (*t).clone()).collect(),
        domains: domains.iter().map(|d| domain_id(d)).collect(),
        properties,
        lifecycle,
        visibility: Visibility::Public,
        provenance: Provenance::catalog(package, file),
        comment: comment.map(str::to_owned),
        id,
    }
}

fn prop(local: &str, ty: TypeId, pkg: &PackageId) -> PropertyDecl {
    PropertyDecl {
        id: PropertyId::catalog(pkg, local),
        ty: TypeRef::Bound { id: ty },
        multiplicity: Some(Multiplicity {
            lower: 0,
            upper: Some(1),
        }),
        default_value: None,
        visibility: Visibility::Public,
    }
}

fn pkg(
    id: PackageId,
    version: &str,
    imports: Vec<PackageId>,
    role: PackageRole,
) -> OntologyPackage {
    let release = PackageReleaseRef::bootstrap(id.clone(), version);
    OntologyPackage {
        ontology_id: id,
        ontology_version: version.to_owned(),
        source_revision: None,
        release,
        imports,
        role,
        fingerprint: crate::fingerprint::ContentDigest::hash_bytes(b""),
    }
}

/// Person split used by evolution tests.
pub fn person_split() -> RefactoringMap {
    RefactoringMap {
        from: t_core("Person"),
        into: vec![
            t_core("Individual"),
            t_core("Organization"),
            t_core("UnallocatedPerson"),
        ],
    }
}

fn build_core_snapshot(thing_comment: Option<&str>) -> OntologySnapshot {
    let thing = t_core("Thing");
    let abstract_thing = t_core("AbstractThing");
    let physical = t_core("PhysicalThing");
    let information = t_core("InformationThing");
    let behavioural = t_core("BehaviouralThing");
    let unallocated_thing = t_core("UnallocatedThing");
    let mass_bearing = t_core("MassBearing");
    let person = t_core("Person");
    let individual = t_core("Individual");
    let organization = t_core("Organization");
    let unallocated_person = t_core("UnallocatedPerson");
    let boolean = t_kerml("Boolean");
    let integer = t_kerml("Integer");
    let real = t_kerml("Real");
    let string = t_kerml("String");
    let mass_q = t_phys("Mass");
    let force = t_phys("Force");
    let energy = t_phys("Energy");
    let power = t_phys("Power");
    let temperature = t_phys("Temperature");
    let pressure = t_phys("Pressure");
    let time = t_phys("Time");
    let length = t_phys("Length");
    let electric_motor = t_el("ElectricMotor");
    let servo = t_el("ServoMotor");
    let deprecated = t_core("LegacyGadget");
    let replacement = t_core("Gadget");

    let mut b = OntologySnapshotBuilder::new()
        .source_kind(OntologySourceKind::Bootstrap)
        .package(pkg(
            PackageId::omg_kerml(),
            crate::versions::KERML_STDLIB_SUBSET_VERSION,
            vec![],
            PackageRole::StandardLibrary,
        ))
        .package(pkg(
            PackageId::os20_core(),
            crate::versions::OS20_CORE_ONTOLOGY_VERSION,
            vec![PackageId::omg_kerml()],
            PackageRole::Ontology,
        ))
        .package(pkg(
            PackageId::os20_physics(),
            "0.1.0",
            vec![PackageId::os20_core(), PackageId::omg_kerml()],
            PackageRole::Ontology,
        ))
        .package(pkg(
            PackageId::os20_electrical(),
            "0.1.0",
            vec![PackageId::os20_core(), PackageId::os20_physics()],
            PackageRole::Ontology,
        ));

    for (local, name) in baseline_domains() {
        b = b.domain(SemanticDomain {
            id: domain_id(local),
            name: name.to_owned(),
            defined_in: PackageId::os20_core(),
        });
    }

    b = b
        .type_decl(type_decl(
            boolean.clone(),
            OntologyTypeKind::DataType,
            &[],
            &[],
            &[],
            vec![],
            "ScalarValues.kerml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            integer.clone(),
            OntologyTypeKind::DataType,
            &[],
            &[],
            &[],
            vec![],
            "ScalarValues.kerml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            real.clone(),
            OntologyTypeKind::DataType,
            &[],
            &[],
            &[],
            vec![],
            "ScalarValues.kerml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            string.clone(),
            OntologyTypeKind::DataType,
            &[],
            &[],
            &[],
            vec![],
            "ScalarValues.kerml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            thing.clone(),
            OntologyTypeKind::Classifier,
            &[],
            &[],
            &["Design"],
            vec![],
            "Thing.sysml",
            thing_comment,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            abstract_thing.clone(),
            OntologyTypeKind::Classifier,
            &[&thing],
            &[],
            &["Design"],
            vec![],
            "Thing.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            mass_bearing.clone(),
            OntologyTypeKind::Classifier,
            &[],
            &[],
            &["Physics"],
            vec![prop("mass", mass_q.clone(), &PackageId::os20_core())],
            "Mixins.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            physical.clone(),
            OntologyTypeKind::Classifier,
            &[&thing],
            &[&mass_bearing],
            &["Structure", "Physics"],
            vec![
                prop("position", length.clone(), &PackageId::os20_core()),
                prop("orientation", real.clone(), &PackageId::os20_core()),
                prop("geometry", t_core("GeometryBody"), &PackageId::os20_core()),
                prop("material", string.clone(), &PackageId::os20_core()),
            ],
            "PhysicalThing.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            t_core("GeometryBody"),
            OntologyTypeKind::Structure,
            &[&abstract_thing],
            &[],
            &["Geometry"],
            vec![],
            "Geometry.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            information.clone(),
            OntologyTypeKind::Classifier,
            &[&thing],
            &[],
            &["Design"],
            vec![],
            "Thing.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            behavioural.clone(),
            OntologyTypeKind::Classifier,
            &[&thing],
            &[],
            &["Behaviour"],
            vec![],
            "Thing.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            unallocated_thing.clone(),
            OntologyTypeKind::Classifier,
            &[&thing],
            &[],
            &["Design"],
            vec![],
            "Unallocated.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            person.clone(),
            OntologyTypeKind::Classifier,
            &[&thing],
            &[],
            &["Design"],
            vec![],
            "Person.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            individual.clone(),
            OntologyTypeKind::Classifier,
            &[&person],
            &[],
            &["Design"],
            vec![],
            "Person.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            organization.clone(),
            OntologyTypeKind::Classifier,
            &[&person],
            &[],
            &["Design"],
            vec![],
            "Person.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            unallocated_person.clone(),
            OntologyTypeKind::Classifier,
            &[&person, &unallocated_thing],
            &[],
            &["Design"],
            vec![],
            "Person.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            replacement.clone(),
            OntologyTypeKind::Classifier,
            &[&thing],
            &[],
            &["Design"],
            vec![],
            "Gadget.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            deprecated.clone(),
            OntologyTypeKind::Classifier,
            &[&thing],
            &[],
            &["Design"],
            vec![],
            "Gadget.sysml",
            None,
            LifecycleStatus::Deprecated {
                replacement: Some(replacement.as_element().clone()),
            },
        ));

    // Physics quantities — extension point, no solver.
    for (id, dim) in [
        (mass_q.clone(), "M"),
        (force.clone(), "M L T^-2"),
        (energy.clone(), "M L^2 T^-2"),
        (power.clone(), "M L^2 T^-3"),
        (temperature.clone(), "Θ"),
        (pressure.clone(), "M L^-1 T^-2"),
        (time.clone(), "T"),
        (length.clone(), "L"),
    ] {
        b = b
            .type_decl(type_decl(
                id.clone(),
                OntologyTypeKind::Quantity,
                &[&thing],
                &[],
                &["Physics"],
                vec![],
                "Quantities.sysml",
                None,
                LifecycleStatus::Active,
            ))
            .quantity(QuantityType {
                id: id.clone(),
                dimension: dim.to_owned(),
                preferred_unit: Some(UnitId::catalog(
                    &PackageId::os20_physics(),
                    match dim {
                        "M" => "Kilogram",
                        "L" => "Metre",
                        "T" => "Second",
                        "Θ" => "Kelvin",
                        "M L T^-2" => "Newton",
                        "M L^2 T^-2" => "Joule",
                        "M L^2 T^-3" => "Watt",
                        "M L^-1 T^-2" => "Pascal",
                        _ => "One",
                    },
                )),
                value_type: real.clone(),
            });
    }

    b = b
        .type_decl(type_decl(
            electric_motor.clone(),
            OntologyTypeKind::Classifier,
            &[&physical, &behavioural],
            &[],
            &[
                "Structure",
                "Electrical",
                "Behaviour",
                "Simulation",
                "Manufacturing",
            ],
            vec![prop(
                "inputPower",
                power.clone(),
                &PackageId::os20_electrical(),
            )],
            "Electrical.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .type_decl(type_decl(
            servo.clone(),
            OntologyTypeKind::Classifier,
            &[&electric_motor],
            &[],
            &["Structure", "Electrical", "Behaviour"],
            vec![],
            "Electrical.sysml",
            None,
            LifecycleStatus::Active,
        ))
        .unallocated(UnallocatedPattern {
            of: person.clone(),
            unallocated: unallocated_person.clone(),
        })
        .evolution(EvolutionEdge {
            kind: EvolutionKind::SplitInto,
            from: person.as_element().clone(),
            to: individual.as_element().clone(),
        })
        .evolution(EvolutionEdge {
            kind: EvolutionKind::SplitInto,
            from: person.as_element().clone(),
            to: organization.as_element().clone(),
        })
        .evolution(EvolutionEdge {
            kind: EvolutionKind::SplitInto,
            from: person.as_element().clone(),
            to: unallocated_person.as_element().clone(),
        })
        .evolution(EvolutionEdge {
            kind: EvolutionKind::ReplacedBy,
            from: deprecated.as_element().clone(),
            to: replacement.as_element().clone(),
        });

    let has_mass = PredicateId::catalog(&PackageId::os20_core(), "hasMass");
    b = b.predicate(Predicate {
        id: has_mass.clone(),
        relation: RelationType {
            id: has_mass,
            source_type: physical.clone(),
            target_type: mass_q,
            cardinality: Some(Multiplicity {
                lower: 0,
                upper: Some(1),
            }),
            inverse: None,
            algebra: RelationAlgebra::default(),
            domains: vec![domain_id("Physics")],
        },
    });

    let contains = PredicateId::catalog(&PackageId::os20_core(), "contains");
    b = b.predicate(Predicate {
        id: contains.clone(),
        relation: RelationType {
            id: contains,
            source_type: thing.clone(),
            target_type: thing,
            cardinality: None,
            inverse: None,
            algebra: RelationAlgebra {
                symmetric: false,
                transitive: true,
            },
            domains: vec![domain_id("Structure")],
        },
    });

    b.build()
}

/// KerML scalar types mapped from the vendored library (not OS20-invented names).
pub fn kerml_scalars() -> [&'static str; 4] {
    ["Boolean", "Integer", "Real", "String"]
}
