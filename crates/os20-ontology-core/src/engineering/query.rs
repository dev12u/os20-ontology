//! Semantic queries over an engineering graph. Not SQL.

use std::collections::{BTreeSet, VecDeque};

use crate::engineering::graph::EngineeringGraph;
use crate::engineering::relations::CONTAINS;
use crate::identity::TypeId;
use crate::snapshot::OntologySnapshot;

const INSTANCE_OF: &str = "@os20/engineering#instanceOf";
const HAS_GEOMETRY: &str = "@os20/engineering#hasGeometry";
const ALLOCATED_TO: &str = "@os20/engineering#allocatedTo";
const EVIDENCE: &str = "@os20/engineering#providesEvidenceFor";
const VERIFIED_BY: &str = "@os20/engineering#verifiedBy";
const SIMULATES: &str = "@os20/engineering#simulates";
const USES_GEOMETRY: &str = "@os20/engineering#usesGeometry";
const USES_MATERIAL: &str = "@os20/engineering#usesMaterial";
const HAS_PORT: &str = "@os20/engineering#hasPort";
const CONNECTS: &str = "@os20/engineering#connects";
const PRODUCED_BY: &str = "@os20/engineering#producedBy";
const GENERATED_BY: &str = "@os20/engineering#generatedBy";
const PROVIDED_BY: &str = "@os20/engineering#providedBy";
const DERIVED_FROM: &str = "@os20/engineering#derivedFrom";

/// Query selector.
#[derive(Clone, Copy, Debug)]
pub enum Query {
    /// Recursive `contains` closure.
    RecursiveParts,
    /// Geometry of a component.
    GeometryOf,
    /// Requirements allocated to a part.
    AllocatedRequirements,
    /// Evidence supporting a requirement.
    EvidenceFor,
    /// Tests that verify a requirement.
    TestsVerifying,
    /// Simulations that use a geometry.
    SimulationsUsingGeometry,
    /// Simulations that use a material.
    SimulationsUsingMaterial,
    /// Reverse impact of changing a component.
    ImpactOf,
    /// Package that provides a definition.
    PackageProvides,
    /// Instances of a definition.
    InstancesOf,
    /// Ports / interfaces of a component.
    PublicInterfaces,
    /// Electrically connected components.
    ElectricallyConnected,
    /// Information-exchanging components.
    InformationConnected,
    /// Manufacturing operation for a feature.
    FeatureProducedBy,
    /// Evidence that produced a validation/test result.
    EvidenceProducingResult,
    /// Results marked inconclusive.
    InconclusiveAssurance,
    /// Artifact/tool/run that produced a result.
    ProducedByActivity,
    /// Property origins (measured/calculated/simulated).
    PropertyOrigins,
    /// Requirements without supporting evidence.
    UnsatisfiedRequirements,
    /// Dependency closure (`contains` + `derivedFrom` + `instanceOf`).
    DependencyClosure,
}

/// Query result ids (stable, sorted).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QueryAnswer {
    /// Matching element ids.
    pub ids: Vec<String>,
}

/// Run a semantic query from `start` (when required).
pub fn query(
    snapshot: &OntologySnapshot,
    graph: &EngineeringGraph,
    q: Query,
    start: &str,
) -> QueryAnswer {
    let mut ids = match q {
        Query::RecursiveParts => walk(graph, start, CONTAINS, true),
        Query::GeometryOf => targets(graph, start, HAS_GEOMETRY),
        Query::AllocatedRequirements => sources(graph, start, ALLOCATED_TO),
        Query::EvidenceFor => sources(graph, start, EVIDENCE),
        Query::TestsVerifying => targets(graph, start, VERIFIED_BY),
        Query::SimulationsUsingGeometry => sources(graph, start, USES_GEOMETRY),
        Query::SimulationsUsingMaterial => sources(graph, start, USES_MATERIAL),
        Query::ImpactOf => impact(graph, start),
        Query::PackageProvides => package_provides(graph, start),
        Query::InstancesOf => instances_of(graph, start),
        Query::PublicInterfaces => targets(graph, start, HAS_PORT),
        Query::ElectricallyConnected => connected_kind(graph, start, "electrical"),
        Query::InformationConnected => connected_kind(graph, start, "data"),
        Query::FeatureProducedBy => targets(graph, start, PRODUCED_BY),
        Query::EvidenceProducingResult => sources(graph, start, GENERATED_BY)
            .into_iter()
            .chain(targets(graph, start, GENERATED_BY))
            .collect(),
        Query::InconclusiveAssurance => inconclusive(graph),
        Query::ProducedByActivity => {
            let mut v = sources(graph, start, "@os20/engineering#producesResult");
            v.extend(targets(graph, start, GENERATED_BY));
            v
        }
        Query::PropertyOrigins => property_origins(graph, start),
        Query::UnsatisfiedRequirements => unsatisfied(graph, snapshot),
        Query::DependencyClosure => {
            let mut s = walk(graph, start, CONTAINS, true);
            s.extend(walk(graph, start, DERIVED_FROM, true));
            s.extend(targets(graph, start, INSTANCE_OF));
            s
        }
    };
    ids.sort();
    ids.dedup();
    QueryAnswer { ids }
}

fn targets(graph: &EngineeringGraph, start: &str, pred: &str) -> Vec<String> {
    graph
        .edges
        .iter()
        .filter(|e| e.source.as_str() == start && e.predicate.as_str() == pred)
        .map(|e| e.target.as_str().to_owned())
        .collect()
}

fn sources(graph: &EngineeringGraph, start: &str, pred: &str) -> Vec<String> {
    graph
        .edges
        .iter()
        .filter(|e| e.target.as_str() == start && e.predicate.as_str() == pred)
        .map(|e| e.source.as_str().to_owned())
        .collect()
}

fn walk(graph: &EngineeringGraph, start: &str, pred: &str, skip_self: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut q = VecDeque::from([start.to_owned()]);
    let mut seen = BTreeSet::from([start.to_owned()]);
    while let Some(n) = q.pop_front() {
        if !(skip_self && n == start) {
            out.push(n.clone());
        }
        for e in graph
            .edges
            .iter()
            .filter(|e| e.source.as_str() == n && e.predicate.as_str() == pred)
        {
            let t = e.target.as_str().to_owned();
            if seen.insert(t.clone()) {
                q.push_back(t);
            }
        }
    }
    if skip_self {
        out.retain(|id| id != start);
    }
    out
}

fn impact(graph: &EngineeringGraph, start: &str) -> Vec<String> {
    let mut ids = sources(graph, start, CONTAINS);
    ids.extend(sources(graph, start, HAS_GEOMETRY));
    ids.extend(sources(graph, start, USES_GEOMETRY));
    ids.extend(sources(graph, start, ALLOCATED_TO));
    ids.extend(sources(graph, start, SIMULATES));
    ids.extend(walk(graph, start, CONTAINS, true));
    ids
}

fn package_provides(graph: &EngineeringGraph, start: &str) -> Vec<String> {
    let mut ids = targets(graph, start, PROVIDED_BY);
    if let Some(n) = graph.node(start) {
        if let Some(p) = &n.providing_package {
            ids.push(p.clone());
        }
    }
    ids
}

fn instances_of(graph: &EngineeringGraph, start: &str) -> Vec<String> {
    graph
        .edges
        .iter()
        .filter(|e| e.predicate.as_str() == INSTANCE_OF && e.target.as_str() == start)
        .map(|e| e.source.as_str().to_owned())
        .chain(graph.nodes.values().filter_map(|n| {
            if n.definition.as_ref().is_some_and(|d| d.as_str() == start) {
                Some(n.id.as_str().to_owned())
            } else {
                None
            }
        }))
        .collect()
}

fn connected_kind(graph: &EngineeringGraph, start: &str, kind: &str) -> Vec<String> {
    let ports: BTreeSet<String> = targets(graph, start, HAS_PORT).into_iter().collect();
    let mut peer_ports = BTreeSet::new();
    for e in graph
        .edges
        .iter()
        .filter(|e| e.predicate.as_str() == CONNECTS)
    {
        let a = e.target.as_str();
        if ports.contains(a) {
            for e2 in graph
                .edges
                .iter()
                .filter(|x| x.predicate.as_str() == CONNECTS && x.source == e.source)
            {
                peer_ports.insert(e2.target.as_str().to_owned());
            }
        }
    }
    let mut comps = Vec::new();
    for p in peer_ports {
        if let Some(port) = graph.node(&p) {
            if port.flow_kind.as_deref() != Some(kind) && kind != "data" {
                continue;
            }
            if kind == "data" && port.flow_kind.as_deref() != Some("data") {
                continue;
            }
        }
        comps.extend(sources(graph, &p, HAS_PORT));
    }
    comps.retain(|c| c != start);
    comps
}

fn inconclusive(graph: &EngineeringGraph) -> Vec<String> {
    graph
        .nodes
        .values()
        .filter(|n| {
            matches!(
                n.properties.get("assurance"),
                Some(crate::engineering::graph::EngineeringPropertyValue::Enum { value })
                    if value == "inconclusive"
            )
        })
        .map(|n| n.id.as_str().to_owned())
        .collect()
}

fn property_origins(graph: &EngineeringGraph, start: &str) -> Vec<String> {
    let Some(n) = graph.node(start) else {
        return vec![];
    };
    n.properties
        .iter()
        .filter_map(|(k, v)| match v {
            crate::engineering::graph::EngineeringPropertyValue::Quantity { origin, .. } => {
                Some(format!("{k}:{origin}"))
            }
            _ => None,
        })
        .collect()
}

fn unsatisfied(graph: &EngineeringGraph, snapshot: &OntologySnapshot) -> Vec<String> {
    let req = TypeId::new("@os20/assurance#RequirementDefinition").expect("req");
    graph
        .nodes
        .values()
        .filter(|n| snapshot.is_subtype_of(&n.ty, &req) && !n.is_instance)
        .filter(|n| {
            !graph
                .edges
                .iter()
                .any(|e| e.predicate.as_str() == EVIDENCE && e.target == n.id)
        })
        .map(|n| n.id.as_str().to_owned())
        .collect()
}
