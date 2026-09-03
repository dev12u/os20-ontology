# ADR 0006 — Ontology fingerprint semantics

## Status

Accepted

## Decision

Distinguish `ontology_source_fingerprint` (pins, Git, comments, relative paths) from `ontology_semantic_fingerprint` (canonical declarations). Both are order-independent SHA-256. Neither includes SQLite row ids or absolute paths.
