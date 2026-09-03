# Compatibility

Assignability is **not** an ambiguous “compatible” flag:

| API | Meaning |
| --- | --- |
| `is_same_type` | identical `TypeId` |
| `is_subtype_of` | Specialize DAG (or equal); mixins ignored |
| `is_assignable_to` | subtype **and** quantity-dimension agreement |

`ServoMotor` → `ElectricMotor` / `PhysicalThing`: yes. Reverse: no.

Quantities: `Mass` cannot be assigned to `Length`; `Voltage` cannot be assigned to `Temperature` (`OS20-E4014`) even though both specialize `Thing`.

Feature redefinition follows KerML **covariance**: the redefining type must specialize the original feature type (`OS20-E4012` otherwise). The property **merge engine ranking** is unchanged; ontology only validates types.

Literals stay outside ontology. Constraints may be described; there is no general solver.
