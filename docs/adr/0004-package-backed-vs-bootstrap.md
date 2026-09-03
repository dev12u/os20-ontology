# ADR 0004 — Package-backed runtime vs bootstrap

## Status

Accepted

## Decision

Keep `MemoryOntology::core()` as bootstrap. `OntologyRuntime` / `OntologySnapshot` are the production in-memory model. Package-backed snapshots are constructed from resolver/lock pins (Prompt 2 compiles Git packages into that model). Bootstrap is not deleted.
