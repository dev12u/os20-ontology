# Type system

Ontology types map to KerML/SysML categories: Classifier, DataType, Structure, ValueType, Enumeration, Quantity, Unit, QuantityDimension, Predicate, RelationType.

OS20 does **not** fork KerML. Vendored scalars live under `@omg/kerml` (`Boolean`, `Integer`, `Real`, `String`). OS20 ontology packages add engineering meaning (`@os20/core#Thing`).

## Specialize / Mixin / Extend / Specify / Compose

Reuse the frozen relation registry. Ontology does not implement a second inheritance engine.

- **Specialize** — subtyping; multiple specialization is allowed; iteration is sorted; no arbitrary winner.
- **Mixin** — feature contribution only. Mixin membership is **not** subtype ancestry.
- **Extend** — additive, non-subtyping.
- **Specify** — constrain/describe.
- **Compose** — composition, not inheritance.

Core kinds: Specialize, Declares, Mixin, Extend, Specify, Compose, FeatureTyping, Redefines, Subset, Connect, Reference, Flow, Allocate, Succession, Verify, Validate, Depend.

Custom predicates dispatch to **registered Rust handlers**, never SQL triggers.

## Compatibility

Prefer precise names:

- `is_subtype_of(ty, ancestor)` — Specialize closure (or equality).
- `is_assignable_to(value, target)` — `value` may be used where `target` is required (currently: subtype-or-equal).

Direction: `ServoMotor` is assignable **to** `PhysicalThing`; not the reverse.

No name matching (`type.name == other.name`). Unresolved types are preserved; they are not coerced to `Any`. A universal top type is not introduced as an error hatch (KerML `Anything` remains language stdlib, not OS20 `Thing`).

Cycles emit `SpecializationCycle` diagnostics and queries terminate.

## Properties

Bound features reference `TypeId` (`TypeRef::Bound`). Multiplicity is KerML/SysML multiplicity from the frontend. Defaults are distinct from declared / override / calculated values. Effective properties are **computed** through Specialize + Declares (+ mixin contribution), not stored as a second canonical graph.
