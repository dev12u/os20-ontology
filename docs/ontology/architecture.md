# Ontology architecture

OS20 ontology is a **semantic model**, not a database. Canonical definitions are authored in KerML/SysML inside ordinary Git-backed OS20 packages (`PackageRole::Ontology`). The language frontend produces a `SourceGraph`; ontology interpretation builds an immutable [`OntologySnapshot`](../../crates/os20-ontology-core/src/snapshot.rs). Product surfaces consume `sdk.ontology()`.

```text
Git-authored KerML / SysML
        ↓
  language frontend
        ↓
   SourceGraph
        ↓
ontology interpretation   ← Prompt 2 compiles packages
        ↓
  OntologyRuntime
        ↓
  SemanticResolver
        ↓
  product surfaces
```

Canonical public contract: [ontology-spec-v1.md](../ontology-spec-v1.md). Index: [README.md](README.md).

Git remains authority. SQLite / `.os20` indexes are **rebuildable projections**. Database row IDs are never semantic identity.

`MemoryOntology::core()` remains the bootstrap used by the SDK today. Package-backed snapshots are the migration path: lock pins (`PackageRole::Ontology`) become `OntologyPackage` records without a second package manager.

There is **no process-global ontology**. Each `Os20Sdk` instance holds its own `OntologyRuntime`. Two workspaces may resolve `@os20/physics@1` and `@os20/physics@2` in one process.

This crate does not implement AuthZ, X.509, registry protocol, or a second dependency resolver.
