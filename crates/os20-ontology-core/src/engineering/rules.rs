//! Machine-checkable engineering graph rules. Does not replace frozen validation.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::{DiagnosticCode, OntologyDiagnostic};
use crate::engineering::graph::EngineeringGraph;
use crate::engineering::relations::{CONTAINS, RelationAuthorship, canonical_relations};
use crate::identity::{ElementId, TypeId};
use crate::snapshot::OntologySnapshot;
use crate::versions::ENGINEERING_GRAPH_SCHEMA;

const INSTANCE_OF: &str = "@os20/engineering#instanceOf";
const CONNECTS: &str = "@os20/engineering#connects";
const SELECTS: &str = "@os20/engineering#selects";
const INCOMPATIBLE: &str = "@os20/engineering#incompatibleWith";
const EXCLUDES: &str = "@os20/engineering#excludes";
const PRODUCES: &str = "@os20/engineering#producesResult";
const GENERATED_BY: &str = "@os20/engineering#generatedBy";
const PROVIDED_BY: &str = "@os20/engineering#providedBy";
const REPRESENTED_BY: &str = "@os20/engineering#representedBy";

const DEF: &str = "@os20/engineering#EngineeringDefinition";
const INST: &str = "@os20/engineering#EngineeringInstance";
const SIM_RUN: &str = "@os20/analysis#SimulationRun";
const SIM_RESULT: &str = "@os20/analysis#SimulationResult";
const TEST_EXEC: &str = "@os20/assurance#TestExecution";
const TEST_RESULT: &str = "@os20/assurance#TestResult";

/// Validate an engineering graph against the compiled ontology snapshot.
pub fn validate_engineering_graph(
    snapshot: &OntologySnapshot,
    graph: &EngineeringGraph,
) -> Vec<OntologyDiagnostic> {
    let mut out = Vec::new();
    if graph.schema != ENGINEERING_GRAPH_SCHEMA {
        out.push(OntologyDiagnostic::new(
            DiagnosticCode::UnknownEngineeringSchema,
            format!(
                "unknown engineering graph schema {} (expected {ENGINEERING_GRAPH_SCHEMA})",
                graph.schema
            ),
        ));
        return out;
    }

    let def_ty = TypeId::new(DEF).expect("def");
    let inst_ty = TypeId::new(INST).expect("inst");
    let sim_run = TypeId::new(SIM_RUN).expect("run");
    let sim_result = TypeId::new(SIM_RESULT).expect("sres");
    let test_exec = TypeId::new(TEST_EXEC).expect("tex");
    let test_result = TypeId::new(TEST_RESULT).expect("tres");

    for node in graph.nodes.values() {
        if snapshot.type_summary(&node.ty).is_none() {
            out.push(dangling(node.id.as_str(), "type"));
        }
        if let Some(d) = &node.definition {
            if snapshot.type_summary(d).is_none() {
                out.push(dangling(d.as_str(), "definition"));
            }
        }
        if node.is_instance {
            if !snapshot.is_subtype_of(&node.ty, &inst_ty) {
                out.push(OntologyDiagnostic::new(
                    DiagnosticCode::InstanceOfNotDefinition,
                    format!(
                        "`{}` is marked instance but type `{}` is not an EngineeringInstance",
                        node.id, node.ty
                    ),
                ));
            }
            if let Some(d) = &node.definition {
                if !snapshot.is_subtype_of(d, &def_ty) {
                    out.push(OntologyDiagnostic::new(
                        DiagnosticCode::InstanceOfNotDefinition,
                        format!(
                            "`{}` instanceOf target `{}` is not an EngineeringDefinition",
                            node.id, d
                        ),
                    ));
                }
            }
            if snapshot.is_subtype_of(&node.ty, &def_ty)
                && !snapshot.is_subtype_of(&node.ty, &inst_ty)
            {
                out.push(OntologyDiagnostic::new(
                    DiagnosticCode::InstanceSpecializesDefinition,
                    format!("`{}` uses a definition type as the instance class", node.id),
                ));
            }
        }
        if let Some(pv) = node.properties.values().find_map(|p| match p {
            crate::engineering::graph::EngineeringPropertyValue::Quantity {
                quantity_kind,
                dimension,
                ..
            } => Some((quantity_kind.as_str(), dimension.as_str())),
            _ => None,
        }) {
            if let Ok(kid) = TypeId::new(pv.0) {
                if let Some(q) = snapshot.quantity(&kid) {
                    if q.dimension != pv.1 {
                        out.push(OntologyDiagnostic::new(
                            DiagnosticCode::QuantityDimensionMismatch,
                            format!(
                                "`{}` quantity kind `{}` has dimension `{}` but value uses `{}`",
                                node.id, pv.0, q.dimension, pv.1
                            ),
                        ));
                    }
                }
            }
        }
    }

    let mut contains: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for e in &graph.edges {
        let pred = e.predicate.as_str();
        if graph.node(e.source.as_str()).is_none() {
            out.push(dangling(e.source.as_str(), "edge source"));
        }
        if graph.node(e.target.as_str()).is_none() {
            out.push(dangling(e.target.as_str(), "edge target"));
        }
        if let Some(decl) = canonical_relations().iter().find(|r| r.id == pred)
            && decl.authorship == RelationAuthorship::Derived
        {
            out.push(OntologyDiagnostic::new(
                DiagnosticCode::MalformedOntologySource,
                format!("derived relation `{pred}` must not be authored"),
            ));
        }
        if pred == INSTANCE_OF {
            if let Some(src) = graph.node(e.source.as_str()) {
                if !src.is_instance {
                    out.push(OntologyDiagnostic::new(
                        DiagnosticCode::InstanceSpecializesDefinition,
                        format!("`{}` has instanceOf but is not an instance", src.id),
                    ));
                }
            }
            if let Some(tgt) = graph.node(e.target.as_str()) {
                if tgt.is_instance || !snapshot.is_subtype_of(&tgt.ty, &def_ty) {
                    out.push(OntologyDiagnostic::new(
                        DiagnosticCode::InstanceOfNotDefinition,
                        format!("instanceOf target `{}` is not a definition node", e.target),
                    ));
                }
            }
        }
        if pred == CONTAINS {
            contains
                .entry(e.source.as_str().into())
                .or_default()
                .push(e.target.as_str().into());
        }
        if pred == CONNECTS {
            // collected below for pair-wise port checks
        }
        if pred == PROVIDED_BY {
            if let Some(src) = graph.node(e.source.as_str()) {
                if snapshot.type_summary(&src.ty).is_none() {
                    out.push(OntologyDiagnostic::new(
                        DiagnosticCode::MissingPackageExport,
                        format!("providedBy source `{}` is not an exported type", src.id),
                    ));
                }
            }
        }
        if pred == REPRESENTED_BY {
            if let Some(tgt) = graph.node(e.target.as_str()) {
                if tgt.artifact.is_none() {
                    out.push(OntologyDiagnostic::new(
                        DiagnosticCode::ArtifactRoleMismatch,
                        format!(
                            "`{}` is representedBy `{}` but that node has no ArtifactDigest",
                            e.source, e.target
                        ),
                    ));
                }
            }
        }
    }

    if let Some(cycle) = cycle(&contains) {
        out.push(OntologyDiagnostic::new(
            DiagnosticCode::CompositionCycle,
            format!("contains cycle involving `{cycle}`"),
        ));
    }

    check_ports(graph, snapshot, &mut out);
    check_configuration(graph, &mut out);
    check_sim_test(
        graph,
        snapshot,
        &sim_run,
        &sim_result,
        &test_exec,
        &test_result,
        &mut out,
    );
    out
}

fn dangling(id: &str, role: &str) -> OntologyDiagnostic {
    OntologyDiagnostic::new(
        DiagnosticCode::DanglingSemanticId,
        format!("dangling {role} `{id}`"),
    )
    .with_element(ElementId::new(id).unwrap_or_else(|_| {
        ElementId::new("@os20/engineering#EngineeringThing").expect("fallback")
    }))
}

fn cycle(adj: &BTreeMap<String, Vec<String>>) -> Option<String> {
    fn dfs(
        n: &str,
        adj: &BTreeMap<String, Vec<String>>,
        stack: &mut BTreeSet<String>,
        seen: &mut BTreeSet<String>,
    ) -> Option<String> {
        if !stack.insert(n.to_owned()) {
            return Some(n.to_owned());
        }
        if seen.insert(n.to_owned()) {
            for c in adj.get(n).into_iter().flatten() {
                if let Some(hit) = dfs(c, adj, stack, seen) {
                    return Some(hit);
                }
            }
        }
        stack.remove(n);
        None
    }
    let mut seen = BTreeSet::new();
    for k in adj.keys() {
        let mut stack = BTreeSet::new();
        if let Some(c) = dfs(k, adj, &mut stack, &mut seen) {
            return Some(c);
        }
    }
    None
}

fn check_ports(
    graph: &EngineeringGraph,
    snapshot: &OntologySnapshot,
    out: &mut Vec<OntologyDiagnostic>,
) {
    let mut by_conn: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for e in graph.outgoing_all(CONNECTS) {
        by_conn
            .entry(e.source.as_str().into())
            .or_default()
            .push(e.target.as_str().into());
    }
    let electrical = TypeId::new("@os20/engineering#ElectricalPort").ok();
    for (_conn, ports) in by_conn {
        if ports.len() < 2 {
            continue;
        }
        for pair in ports.windows(2) {
            let a = graph.node(&pair[0]);
            let b = graph.node(&pair[1]);
            let (Some(a), Some(b)) = (a, b) else { continue };
            if let (Some(ka), Some(kb)) = (&a.flow_kind, &b.flow_kind) {
                if ka != kb {
                    out.push(OntologyDiagnostic::new(
                        DiagnosticCode::PortIncompatible,
                        format!(
                            "ports `{}` and `{}` have incompatible flow kinds `{ka}` vs `{kb}`",
                            a.id, b.id
                        ),
                    ));
                }
            }
            if let (Some(da), Some(db)) = (&a.direction, &b.direction)
                && da != "inout"
                && db != "inout"
                && da == db
            {
                out.push(OntologyDiagnostic::new(
                    DiagnosticCode::PortIncompatible,
                    format!(
                        "ports `{}` and `{}` have the same direction `{da}`",
                        a.id, b.id
                    ),
                ));
            }
            if let (Some(et), Some(a_ty), Some(b_ty)) =
                (electrical.as_ref(), Some(&a.ty), Some(&b.ty))
            {
                let a_el = snapshot.is_subtype_of(a_ty, et);
                let b_el = snapshot.is_subtype_of(b_ty, et);
                // Port instances specialize PortInstance, not PortDefinition.
                // Kind is taken from flow_kind / definition, not the instance class.
                let _ = (a_el, b_el);
            }
        }
    }
}

fn check_configuration(graph: &EngineeringGraph, out: &mut Vec<OntologyDiagnostic>) {
    let mut selected: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for e in graph
        .edges
        .iter()
        .filter(|e| e.predicate.as_str() == SELECTS)
    {
        selected
            .entry(e.source.as_str().into())
            .or_default()
            .insert(e.target.as_str().into());
    }
    let mut bad: BTreeSet<(String, String)> = BTreeSet::new();
    for e in graph
        .edges
        .iter()
        .filter(|e| matches!(e.predicate.as_str(), INCOMPATIBLE | EXCLUDES))
    {
        bad.insert((e.source.as_str().into(), e.target.as_str().into()));
        bad.insert((e.target.as_str().into(), e.source.as_str().into()));
    }
    for (cfg, opts) in selected {
        for a in &opts {
            for b in &opts {
                if a < b && bad.contains(&(a.clone(), b.clone())) {
                    out.push(OntologyDiagnostic::new(
                        DiagnosticCode::ConfigurationIncompatible,
                        format!("configuration `{cfg}` selects incompatible `{a}` and `{b}`"),
                    ));
                }
            }
        }
    }
}

fn check_sim_test(
    graph: &EngineeringGraph,
    snapshot: &OntologySnapshot,
    sim_run: &TypeId,
    sim_result: &TypeId,
    test_exec: &TypeId,
    test_result: &TypeId,
    out: &mut Vec<OntologyDiagnostic>,
) {
    for node in graph.nodes.values() {
        if snapshot.is_subtype_of(&node.ty, sim_result) {
            let has_run = graph.edges.iter().any(|e| {
                e.source == node.id
                    && e.predicate.as_str() == GENERATED_BY
                    && graph
                        .node(e.target.as_str())
                        .is_some_and(|n| snapshot.is_subtype_of(&n.ty, sim_run))
            }) || graph.edges.iter().any(|e| {
                e.target == node.id
                    && e.predicate.as_str() == PRODUCES
                    && graph
                        .node(e.source.as_str())
                        .is_some_and(|n| snapshot.is_subtype_of(&n.ty, sim_run))
            });
            if !has_run {
                out.push(OntologyDiagnostic::new(
                    DiagnosticCode::SimulationResultMissingRun,
                    format!(
                        "simulation result `{}` does not reference a SimulationRun",
                        node.id
                    ),
                ));
            }
        }
        if snapshot.is_subtype_of(&node.ty, test_result) {
            let has_exec = graph.edges.iter().any(|e| {
                e.source == node.id
                    && e.predicate.as_str() == GENERATED_BY
                    && graph
                        .node(e.target.as_str())
                        .is_some_and(|n| snapshot.is_subtype_of(&n.ty, test_exec))
            }) || graph.edges.iter().any(|e| {
                e.target == node.id
                    && e.predicate.as_str() == PRODUCES
                    && graph
                        .node(e.source.as_str())
                        .is_some_and(|n| snapshot.is_subtype_of(&n.ty, test_exec))
            });
            if !has_exec {
                out.push(OntologyDiagnostic::new(
                    DiagnosticCode::TestResultMissingExecution,
                    format!(
                        "test result `{}` does not reference a TestExecution",
                        node.id
                    ),
                ));
            }
        }
    }
}

impl EngineeringGraph {
    fn outgoing_all(
        &self,
        pred: &str,
    ) -> impl Iterator<Item = &crate::engineering::graph::GraphEdge> {
        self.edges
            .iter()
            .filter(move |e| e.predicate.as_str() == pred)
    }
}
