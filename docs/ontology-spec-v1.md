# OS20 Ontology Specification v1

**Status:** Frozen public semantic contract (schema 1). Independent of the Rust crate layout.

This document defines what OS20 ontology *means*. KerML/SysML is the modeling language. Git is authored history. OS20 packages are the distribution unit. The `OntologyRuntime` interprets meaning. CLI, LSP, and Workbench must not redefine it.

## Authority stack

| Layer | Role |
| --- | --- |
| Git | Authored ontology source (KerML/SysML). Immutable commits are source identity. |
| OS20 package | Distribution unit. `PackageRole::Ontology` is reused; there is no second package kind. |
| Registry | Immutable **release metadata** (existing `/v1`). Not an ontology engine. |
| `os20.lock` | Reproducibility boundary: version, repository, `package_root`, commit, tree, manifest digest. |
| SQLite / `.os20` | Rebuildable projection/cache. Never semantic identity. Disposable on format bump. |
| `OntologyRuntime` | Semantic ontology authority **inside** `os20-sdk`. Per SDK instance; never process-global. |
| SemanticResolver | Consumes `OntologyRuntime`; does not replace merge ranking. |
| CLI / LSP / Workbench | Consume SDK/LSP reports only. |

## Identity

Named IDs only. Database row IDs are forbidden.

| Kind | Form | Notes |
| --- | --- | --- |
| Ontology package | `@scope/name` | Version is **not** part of the id. |
| Ontology element / `TypeId` | `@scope/name#Local` | Stable across file moves. Rename changes the named id (documented). |
| `PredicateId` | `@scope/name#Local` | Typed predicate identity. |
| `SemanticDomainId` | `@scope/name#Local` | First-class; not a package and not a quantity dimension. |

`Thing` (`@os20/core#Thing`) is the root of the specialize lattice. Mixins are not supertypes. `Unallocated{T}` is a real classifier, never SQL NULL.

## Version domains (must not be conflated)

1. **Ontology package SemVer** — lock/resolver version of `@os20/core`, `@os20/physics`, …
2. **Ontology semantic fingerprint** — `ontology-semantic-v1` digest of definitions (comments excluded).
3. **Ontology runtime format** — `ONTOLOGY_RUNTIME_FORMAT_VERSION` (currently `1`). Invalidates derived caches.
4. **Language frontend version** — owned by `os20-language`.
5. **Resolver semantics version** — owned by the existing SemanticResolver.
6. **Release descriptor schema** — frozen package-release descriptor (not ontology-specific).

Compiled-in bootstrap core is **`OS20_CORE_ONTOLOGY_VERSION` (`0.1.0`)**. Package-backed `@os20/core` replaces bootstrap types it declares. There is no unversioned mutable global core.

## Type semantics

- Multiple specialization is allowed.
- Mixins ≠ subtypes and do not create assignability.
- Properties use the existing resolver merge; ontology checks override covariance.
- Assignability is specialize-based, never name-based.
- Quantity dimensions (e.g. Mass vs Length) diagnose `OS20-E4014` when incompatible.
- Predicates have typed endpoints; illegal endpoints diagnose `OS20-E4015`.
- Only **built-in Rust** relation handlers execute. Ontology packages reference handlers declaratively; they are not executable scripts.

## Evolution

Deprecation, `ReplacedBy`, `SplitInto`, `MergedInto` are explicit metadata. Historical IDs are never rewritten. A `MigrationReport` lists removed IDs, replacements, splits, unresolved old references, and suggested targets with `autoRewrite: false`.

## Diagnostics

Compiler/runtime codes `OS20-E4001`–`E4015` / `W4001`. Missing ontology does not panic.

## Product JSON

Schema 1 envelopes. Kinds include `ontology_status`, `ontology_type`, `ontology_search`, `ontology_diff`, `ontology_impact`. No DB IDs.

## Out of scope (deferred)

Full units solver, thermodynamic solver, simulation execution, geometry kernel, manufacturing process solver, safety-case reasoning, automated ontology alignment. AuthZ does not define ontology semantics.
