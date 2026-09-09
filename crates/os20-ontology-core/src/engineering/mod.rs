//! OS20 engineering ontology v1 — vocabulary on the frozen identity substrate.
//!
//! In this checkout there is no separate UIDS crate. The frozen OS20 identity
//! protocol (`@scope/name#Local`, Specialize/Mixin/Extend, Git packages, lock,
//! provenance) is the semantic substrate. This module is **engineering meaning**.

mod graph;
mod jsonld;
mod powertrain;
mod query;
mod relations;
mod rules;
mod types;

pub use graph::{
    ArtifactRef, EngineeringGraph, EngineeringNode, EngineeringPropertyValue, GraphEdge,
};
pub use jsonld::{JsonLdDocument, to_jsonld};
pub use powertrain::electric_powertrain_graph;
pub use query::{Query, QueryAnswer, query};
pub use relations::{RelationAuthorship, RelationDecl, canonical_relations, relation, relation_id};
pub use rules::validate_engineering_graph;
pub use types::{
    engineering_compile_request, engineering_graphs, engineering_manifests, engineering_packages,
};

use crate::compiler::{OntologyCompileResult, OntologyCompiler};

/// Compile the v1 engineering ontology packages (locked, offline).
pub fn compile_engineering_v1() -> OntologyCompileResult {
    OntologyCompiler::compile(engineering_compile_request())
}
