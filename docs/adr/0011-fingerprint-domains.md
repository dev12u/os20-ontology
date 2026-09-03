# ADR 0011 — Fingerprint digest domains

## Status

Accepted

## Decision

Ontology fingerprints are SHA-256 of canonical lines **prefixed** by a purpose token:

- `ontology-semantic-v1`
- `ontology-source-v1`
- `ontology-package-v1`

Raw unprefixed digests are not reused across these namespaces.
