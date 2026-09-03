# Ontology compiler

Pipeline:

```text
.kerml / .sysml
     ↓
frozen os20-language   (not reimplemented here)
     ↓
bound SourceGraph
     ↓
OntologyCompiler
     ↓
OntologyPackageSnapshot
     ↓
OntologyRuntime
```

`OntologyCompiler` consumes **bound declarations** (`BoundSourceGraph`): classifiers, datatypes, structures, features, Specialize / Mixin / Extend / Specify, quantities, units, predicates, domain annotations.

Cross-file and cross-package names are resolved with import-scope binding equivalent to the frozen binder (`@pkg#Name` or imported locals). Durable edges are `TypeId`s.

## Language gap

Semantic-domain and quantity-dimension fields that the current language cannot express are **explicit** `BoundElement` metadata (`OntologyMetadataExtension`), not hidden source conventions. See ADR 0010.

## Diagnostics (`OS20-E4xxx` / `W4xxx`)

Does not collide with `E1xxx`/`E2xxx`/`E3xxx`.

| Code | Meaning |
| --- | --- |
| E4001 | duplicate ontology id |
| E4002 | illegal protected redefinition |
| E4003 | unresolved ontology type |
| E4004 | incompatible ontology role |
| E4005 | invalid domain declaration |
| E4006 | evolution mapping target missing |
| E4007 | missing required ontology |
| E4008 | missing Git source offline |
| E4009 | specialization cycle |
| E4010 | ontology package dependency cycle |
| E4011 | malformed bound SourceGraph |
| W4001 | comment-only change |
