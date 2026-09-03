# Physics foundation (quantities)

OS20 represents **quantity types and dimensions** as ontology classifiers (e.g. `@os20/physics#Mass`, length, temperature). This is **not** a units solver, formula engine, or simulation runtime.

- Dimensions are first-class type facts, distinct from semantic domains (`Physics` domain ≠ `Mass` dimension).
- Assigning `Mass` where `Length` is required is `OS20-E4014`.
- Preferred units may be recorded; conversion and thermodynamic closure are **deferred**.

KerML scalars (`@omg/kerml#Real`, …) remain the language standard-library subset. They must not be given `@os20/core#…` identities.
