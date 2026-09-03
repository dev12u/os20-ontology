# Ontology offline

Locked / locked-offline loads ontology source from the **bare Git object cache** using `os20.lock` pins. No working-tree checkout. `FetchPolicy::Never` forbids Git fetch and registry HTTP.

Missing blobs yield `OS20-E4008` (degraded snapshot, no panic, no fake types).

Delete `.os20` and reindex from the same snapshot → same semantic fingerprint.

Online `Update` may resolve versions through the **existing** registry API (no ontology-specific endpoint).
