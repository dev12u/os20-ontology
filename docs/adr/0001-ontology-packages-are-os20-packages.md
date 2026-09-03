# ADR 0001 — Ontology packages are OS20 packages

## Status

Accepted (Prompt 1)

## Decision

Ontology definitions are ordinary immutable Git-backed OS20 packages with `PackageRole::Ontology`. They use the existing resolver, lock, and release model.

## Consequences

No ontology-specific package manager, registry, or import syntax. Cross-ontology references bind to `@package#Name` ids.
