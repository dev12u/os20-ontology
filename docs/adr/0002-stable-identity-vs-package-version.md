# ADR 0002 — Stable identity vs package version

## Status

Accepted

## Decision

Semantic identity (`@os20/core#Thing`) is independent of package release identity (`@os20/core@2.1.0` + commit/tree/manifest). `OntologyPackage` holds a `PackageReleaseRef` rather than inventing a parallel release id.
