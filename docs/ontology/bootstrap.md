# Bootstrap strategy

**In the binary (minimal emergency foundation):**

- `MemoryOntology::core()` catalog version `OS20_CORE_ONTOLOGY_VERSION` (`0.1.0`)
- Built-in `RelationRegistry` Rust handlers
- Protected IDs (`@os20/core#Thing`, `@omg/kerml` scalars)

**Loaded from packages (production path):**

- Locked `PackageRole::Ontology` Git trees (`@os20/core`, physics, electrical, …)
- KerML subset as `@omg/kerml` standard library (not an OS20 ontology package)

Order:

1. KerML standard library (`@omg/kerml` scalars)
2. Minimal OS20 bootstrap (`MemoryOntology::core()`) as **emergency foundation**
3. Resolved ontology packages in deterministic dependency order

`MemoryOntology::core()` is **not** deleted. Package-backed `@os20/core` **replaces** bootstrap types it actually declares. Remaining bootstrap IDs stay until a real package defines them. There is no unversioned mutable global core.

Protected IDs (`@os20/core#Thing`, `@omg/kerml#Real`, …) cannot be redefined by a different package (no same-ID shadowing). Extension is Specialize / Mixin / Extend / Specify only.

Override rule: the declaring package owns the id. Other packages may specialize it, never re-bind it.
