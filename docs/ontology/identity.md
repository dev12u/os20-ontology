# Ontology identity

## Semantic identity

Named ontology elements use durable, inspectable ids:

```text
@os20/core#Thing
@os20/physics#Mass
```

Typed wrappers: `PackageId`, `ElementId`, `TypeId`, `PredicateId`, `PropertyId`, `UnitId`, `SemanticDomainId`. Frozen in [ontology-spec-v1.md](../ontology-spec-v1.md).

`TypeId` is an `ElementId` newtype. Public APIs do not take raw `String` type keys.

Forbidden as public identity: `INTEGER PRIMARY KEY`, `rowid`, SQLite sequences, memory addresses, bare numeric strings.

## Version vs identity

| Concept | Example |
| --- | --- |
| Semantic identity | `Thing` as `@os20/core#Thing` |
| Package release | `@os20/core@2.1.0` plus commit/tree/manifest digest (`PackageReleaseRef`) |
| Ontology revision | the Git commit/tree that supplied the definition |

`OntologyPackage` **references** `PackageReleaseRef`; it does not duplicate release identity.

## Rename

Today, named identity includes the local name. A rename changes `ElementId` unless a future optional stable semantic id is added (ADR 0005). File moves change **provenance** only.

## Provenance

Every declaration stores package, package-relative path, span, commit/tree. Provenance is not identity.
