//! Canonical OS20 ontology model and runtime (ontology track Prompt 1).
//!
//! Git remains source authority. SQLite / derived indexes are rebuildable
//! projections. Database row IDs are never semantic identity.

#![forbid(unsafe_code)]

mod catalog;
mod classify;
mod compatibility;
mod compiler;
mod derived;
mod diagnostics;
mod diff;
mod engineering;
mod evolution;
mod explain;
mod fingerprint;
mod fixtures;
mod identity;
mod impact;
mod incremental;
mod lockfile;
mod merge;
mod model;
mod package;
mod predicate_runtime;
mod refactor;
mod relations;
mod resolver_bridge;
mod runtime;
mod snapshot;
mod source_graph;
mod source_tree;

mod versions;

pub use catalog::{MemoryOntology, baseline_domains, kerml_scalars, person_split};
pub use classify::Classification;
pub use compatibility::{Assignability, AssignabilityReject, QuantityDimension};
pub use compiler::{
    OntologyCompileRequest, OntologyCompileResult, OntologyCompiler, OntologyPackageSnapshot,
    lockfile_from_locked_packages, parse_ontology_manifest,
};
pub use derived::{DerivedIndexError, DerivedOntologyIndex, reindex};
pub use diagnostics::{DiagnosticCode, OntologyDiagnostic, RelatedLocation};
pub use diff::{OntologyChange, OntologyDiff, OntologyDiffKind, SemVerAdvice, ontology_diff};
pub use engineering::{
    ArtifactRef, EngineeringGraph, EngineeringNode, EngineeringPropertyValue, GraphEdge,
    JsonLdDocument, Query, QueryAnswer, RelationAuthorship, RelationDecl, canonical_relations,
    compile_engineering_v1, electric_powertrain_graph, engineering_compile_request,
    engineering_graphs, engineering_manifests, engineering_packages, query, relation, relation_id,
    to_jsonld, validate_engineering_graph,
};
pub use evolution::{
    AbsenceKind, EvolutionEdge, EvolutionKind, RefactoringMap, UnallocatedPattern,
};
pub use explain::{WhyClassified, WhyCompatible, WhyOverride, WhyPropertyType, WhyType};
pub use fingerprint::{ContentDigest, OntologyFingerprints};
pub use fixtures::{
    authored_source, behaviour_graph, behaviour_manifest, core_graph, core_manifest,
    electrical_graph, electrical_manifest, kerml_graph, kerml_manifest, mechanical_graph,
    mechanical_manifest, physics_graph, physics_manifest, physics_v2_graph, physics_v2_manifest,
};
pub use identity::{
    ElementId, IdentityError, OntologyId, PackageId, PredicateId, PropertyId, QuantityTypeId,
    SemanticDomainId, TypeId, UnitId,
};
pub use impact::{
    ImpactReport, ResolvedElementFingerprint, ReverseReferences, invalidate_dependent_caches,
    unrelated_types_preserved,
};
pub use incremental::{InvalidationReport, comment_only_edit, invalidate_from_delta};
pub use lockfile::{LockedPackage, Os20Lockfile};
pub use merge::MergedProperty;
pub use model::{
    EffectiveProperty, OntologyTypeKind, Predicate, PropertyDecl, QuantityType, SemanticDomain,
    TypeDecl, TypeSummary,
};
pub use package::{
    LifecycleStatus, OntologySourceKind, PackageReleaseRef, PackageRole, Provenance, SourceSpan,
    TypeRef, ValueOrigin, Visibility,
};
pub use predicate_runtime::PredicateRuntimeSpec;
pub use refactor::{
    EvolutionGuidance, MigrationReport, OntologyConstraint, ReclassificationPlan,
    migration_analysis, reclassify_semver,
};
pub use relations::{
    Multiplicity, RelationAlgebra, RelationEdge, RelationHandler, RelationInterpretation,
    RelationKind, RelationRegistry, RelationType,
};
pub use resolver_bridge::SemanticResolverOntology;
pub use runtime::{Ontology, OntologyRuntime};
pub use snapshot::{OntologyPackage, OntologySnapshot, OntologySnapshotBuilder};
pub use source_graph::{
    BoundElement, BoundFeature, BoundRelation, BoundSourceGraph, FileComment, MovedDefinition,
    OntologyManifest, OntologyMetadataExtension, SourceGraphDelta,
};
pub use source_tree::{
    FetchPolicy, GitObjectCache, OntologyRegistry, OntologyResolveMode, OntologySourceTree,
    RecordingRegistry, SourceTreeError,
};
pub use versions::{
    ENGINEERING_GRAPH_SCHEMA, FINGERPRINT_PACKAGE, FINGERPRINT_SEMANTIC, FINGERPRINT_SOURCE,
    KERML_STDLIB_SUBSET_VERSION, LANGUAGE_FRONTEND_VERSION_SLOT, MAX_ONTOLOGY_DIAGNOSTICS,
    MAX_SPECIALIZE_VISIT, ONTOLOGY_RUNTIME_FORMAT_VERSION, OS20_CORE_ONTOLOGY_VERSION,
    OS20_ENGINEERING_ONTOLOGY_VERSION, PRODUCT_REPORT_SCHEMA, RELEASE_DESCRIPTOR_SCHEMA_SLOT,
    RESOLVER_SEMANTICS_VERSION_SLOT,
};
