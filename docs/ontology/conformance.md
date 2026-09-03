# Conformance

Tests live in `crates/os20-sdk/tests/prompt5.rs` (plus prompts 1–4). This is **not** a claim of full engineering-ontology coverage.

## Supported

Valid inheritance, multiple specialization, mixin ≠ subtype, property typing, quantity dimension mismatch, predicates with typed endpoints, semantic domains, deprecation metadata, evolution mappings, cross-package specialize, deterministic fingerprints, comment-only semantic no-op, two in-process versions, locked-offline compile, `.os20` rebuild, built-in-only relation handlers, named IDs (no DB ids).

## Partial

Package-backed compile from **bound SourceGraph** (KerML parser remains `os20-language`). SemVer advice for ontology diffs. Incremental invalidation of compatibility caches. Product CLI/LSP/Workbench **wiring** (requires frozen language SDK to link binaries).

## Deferred

Full units/thermodynamic/simulation/geometry/manufacturing/safety solvers; automated ontology alignment; AuthZ as ontology semantics; OMG content beyond the vendored KerML subset.

## Negative corpus

Unresolved type, specialize cycle (`OS20-E4009`, iterative), incompatible override (`E4012`), dimension mismatch (`E4014`), illegal predicate endpoint (`E4015`), duplicate identity (`E4001`), missing evolution target (`E4006`). Malformed bound graphs must not panic.
