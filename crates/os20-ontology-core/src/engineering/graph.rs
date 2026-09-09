//! Engineering instance graph. Types live in the ontology snapshot; this is occurrence.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::identity::{ElementId, PredicateId, TypeId};
use crate::versions::ENGINEERING_GRAPH_SCHEMA;

/// Frozen artifact pointer (bytes stay in the artifact store).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactRef {
    /// Frozen `ArtifactDigest` display form (`artifact:sha256:<hex>`).
    pub digest: String,
    /// Format token (`step`, `scxml`, `fmu`, …) — not an ontology class.
    pub format: String,
    /// Media type if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
}

/// Property payload. Quantity math stays in `os20-quantities`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EngineeringPropertyValue {
    /// Binds a frozen QuantityKind + QuantityValue (strings; engine not duplicated).
    Quantity {
        /// `@os20/physics#Mass` / `@os20/physical#Torque`.
        quantity_kind: String,
        /// Dimension token expected by ontology (`M L^2 T^-2`).
        dimension: String,
        /// Numeric literal.
        value: String,
        /// Unit id or symbol (`@os20/units-v1#newtonMetre` or `N.m`).
        unit: String,
        /// measured | calculated | simulated
        origin: String,
    },
    /// Boolean.
    Boolean {
        /// Value.
        value: bool,
    },
    /// Enumerated token.
    Enum {
        /// Token.
        value: String,
    },
    /// Text (requirement statement, notes).
    Text {
        /// Value.
        value: String,
    },
    /// Named reference.
    Reference {
        /// Target id.
        target: String,
    },
}

/// One semantic object (definition occurrence or instance).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineeringNode {
    /// Stable `@scope/name#Local`.
    pub id: ElementId,
    /// Ontology type of this node (instance class or definition class).
    pub ty: TypeId,
    /// When `is_instance`, the definition this occurrence instantiates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<TypeId>,
    /// True for occurrences; false for definition/export nodes.
    pub is_instance: bool,
    /// Port direction when this node is a port (`in` / `out` / `inout`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Port / flow kind token (`mechanical`, `electrical`, `fluid`, `thermal`, `data`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flow_kind: Option<String>,
    /// Frozen ExecutionRunId (`run:sha256:…`) when this is a run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_run: Option<String>,
    /// Frozen ArtifactDigest when this node *is* an artifact classification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<ArtifactRef>,
    /// Providing package (`@supplier/motor`) — not the engineering object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providing_package: Option<String>,
    /// Named properties.
    #[serde(default)]
    pub properties: BTreeMap<String, EngineeringPropertyValue>,
}

/// Authored edge. Derived inverses are not stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    /// Predicate id.
    pub predicate: PredicateId,
    /// Source node.
    pub source: ElementId,
    /// Target node.
    pub target: ElementId,
}

/// Project-level semantic graph. Not a database.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineeringGraph {
    /// Schema; unknown versions fail closed.
    pub schema: u32,
    /// Nodes by id.
    pub nodes: BTreeMap<String, EngineeringNode>,
    /// Authored edges.
    pub edges: Vec<GraphEdge>,
}

impl Default for EngineeringGraph {
    fn default() -> Self {
        Self {
            schema: ENGINEERING_GRAPH_SCHEMA,
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        }
    }
}

impl EngineeringGraph {
    /// Empty v1 graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a node.
    pub fn insert(&mut self, node: EngineeringNode) {
        self.nodes.insert(node.id.as_str().to_owned(), node);
    }

    /// Authored edge.
    pub fn edge(&mut self, predicate: &str, source: &str, target: &str) {
        self.edges.push(GraphEdge {
            predicate: PredicateId::new(predicate).expect("predicate"),
            source: ElementId::new(source).expect("source"),
            target: ElementId::new(target).expect("target"),
        });
    }

    /// Node by id.
    pub fn node(&self, id: &str) -> Option<&EngineeringNode> {
        self.nodes.get(id)
    }

    /// Outgoing edges of `pred` from `source`.
    pub fn outgoing<'a>(&'a self, source: &str, pred: &str) -> impl Iterator<Item = &'a GraphEdge> {
        let s = source.to_owned();
        let p = pred.to_owned();
        self.edges
            .iter()
            .filter(move |e| e.source.as_str() == s && e.predicate.as_str() == p)
    }

    /// Incoming edges of `pred` to `target`.
    pub fn incoming<'a>(&'a self, target: &str, pred: &str) -> impl Iterator<Item = &'a GraphEdge> {
        let t = target.to_owned();
        let p = pred.to_owned();
        self.edges
            .iter()
            .filter(move |e| e.target.as_str() == t && e.predicate.as_str() == p)
    }
}

impl EngineeringNode {
    /// Definition node (exported reusable semantics).
    pub fn definition(id: &str, ty: &str) -> Self {
        Self {
            id: ElementId::new(id).expect("id"),
            ty: TypeId::new(ty).expect("ty"),
            definition: None,
            is_instance: false,
            direction: None,
            flow_kind: None,
            execution_run: None,
            artifact: None,
            providing_package: None,
            properties: BTreeMap::new(),
        }
    }

    /// Instance node.
    pub fn instance(id: &str, ty: &str, definition: &str) -> Self {
        Self {
            id: ElementId::new(id).expect("id"),
            ty: TypeId::new(ty).expect("ty"),
            definition: Some(TypeId::new(definition).expect("def")),
            is_instance: true,
            direction: None,
            flow_kind: None,
            execution_run: None,
            artifact: None,
            providing_package: None,
            properties: BTreeMap::new(),
        }
    }
}
