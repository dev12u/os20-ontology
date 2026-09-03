# Ontology versioning

Keep these **distinct**:

| Domain | Owner |
| --- | --- |
| Ontology package SemVer | lock / resolver |
| Semantic fingerprint | `ontology-semantic-v1` |
| Runtime format | `ONTOLOGY_RUNTIME_FORMAT_VERSION` (`1`) |
| Language frontend | `os20-language` |
| Resolver semantics | existing SemanticResolver |
| Release descriptor schema | frozen package release |

Compiled-in core catalog: `OS20_CORE_ONTOLOGY_VERSION` = `0.1.0`.

Package versions follow the existing solver: one version per package name in a workspace. Conflicts are not solved inside the ontology compiler.

Two `Os20Sdk` instances may hold `@os20/physics@0.1.0` and `@os20/physics@2.0.0` in one process (no process-global tables).

A major incompatible ontology release is a new package version whose public types differ (e.g. `Mass` removed). SemVer advice remains the existing semantic-diff machinery (not reimplemented here).

Incremental: `SourceGraphDelta` → affected type ids → compatibility invalidation for changed parents and their subtypes. Comment-only edits are a semantic no-op. File moves keep `@package#Name`. Rename follows current named identity (local name is part of the id).
