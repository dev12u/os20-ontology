//! Frozen version domains for ontology. These are **not** interchangeable.

/// In-memory [`crate::OntologySnapshot`] / derived-index format.
///
/// Bump this when the snapshot layout changes. Derived `.os20` projections with
/// a different value are discarded and rebuilt. Semantic truth is never migrated
/// by rewriting SQLite rows.
pub const ONTOLOGY_RUNTIME_FORMAT_VERSION: u32 = 1;

/// SemVer of the compiled-in [`crate::MemoryOntology::core()`] catalog.
///
/// Package-backed `@os20/core` releases replace this bootstrap when locked.
/// There is no unversioned mutable global core.
pub const OS20_CORE_ONTOLOGY_VERSION: &str = "0.1.0";

/// KerML scalar subset version shipped as `@omg/kerml` (not an OS20 ontology package).
pub const KERML_STDLIB_SUBSET_VERSION: &str = "1.0.0";

/// JSON product report envelope (CLI / LSP custom protocol). Independent of runtime format.
pub const PRODUCT_REPORT_SCHEMA: u32 = 1;

/// Release descriptor schema is owned by the frozen package-release model (not ontology).
pub const RELEASE_DESCRIPTOR_SCHEMA_SLOT: u32 = 1;

/// Language frontend version is owned by `os20-language`, not this crate.
pub const LANGUAGE_FRONTEND_VERSION_SLOT: &str = "os20-language";

/// Resolver merge/ranking version is owned by the existing SemanticResolver.
pub const RESOLVER_SEMANTICS_VERSION_SLOT: &str = "os20-sdk-resolver";

/// Digest domain for semantic ontology fingerprints (definitions, no comments).
pub const FINGERPRINT_SEMANTIC: &str = "ontology-semantic-v1";

/// Digest domain for source fingerprints (pins, revisions, comments).
pub const FINGERPRINT_SOURCE: &str = "ontology-source-v1";

/// Digest domain for per-package semantic fingerprints.
pub const FINGERPRINT_PACKAGE: &str = "ontology-package-v1";

/// Upper bound on specialize BFS visits (cycles already use a seen-set).
///
/// 10k-type synthetic graphs stay well under this. Not a modeling prohibition.
pub const MAX_SPECIALIZE_VISIT: usize = 1_048_576;

/// Cap stored diagnostics so a hostile package cannot unbounded-allocate reports.
pub const MAX_ONTOLOGY_DIAGNOSTICS: usize = 16_384;
