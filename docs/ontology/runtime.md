# Ontology runtime

`OntologyRuntime` **supplies** type knowledge to the existing SemanticResolver. It does not replace merge ranking, lock, or binding.

Queries: `is_same_type`, `is_subtype_of`, `is_supertype_of`, `is_assignable_to`, `least_common_supertype` (unique DAG meet only). Mixin membership is not subtyping.

Classification (`classify`, `is_physical`, `semantic_domains`) uses Specialize ancestry, never names. `Thing` is the engineering root when present.

Resolved-element fingerprints continue to mix the ontology semantic fingerprint. Cache invalidation is keyed by that fingerprint and affected type ids — unrelated packages stay valid.
