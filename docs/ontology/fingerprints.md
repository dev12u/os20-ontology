# Ontology fingerprints

Caches mix an ontology fingerprint. Prompt 1 distinguishes:

| Fingerprint | Includes | Excludes |
| --- | --- | --- |
| `ontology_source_fingerprint` | package pins, versions, commit/tree, manifest digest, comments, relative source paths | SQLite row ids, absolute machine paths |
| `ontology_semantic_fingerprint` | canonical type/predicate/property/specialize/mixin/lifecycle/visibility payload | comments, provenance formatting |

Both hashes are SHA-256 over **sorted unique lines**, namespaced by purpose:

- `ontology-semantic-v1`
- `ontology-source-v1`
- `ontology-package-v1`

Do not reuse a raw digest namespace across these domains.

Lock machinery already fingerprints ordered ontology package pins; that is the source-level **set** fingerprint. Semantic fingerprint is computed from declarations so a comment-only Git edit can change source without changing meaning.

Snapshot identity **is** this deterministic pair. No global singleton.

Derived `.os20` indexes store the same named ids and fingerprints. Delete `.os20` and reindex → same semantic fingerprint.
