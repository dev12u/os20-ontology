# Properties and quantities

`EngineeringProperty` with subtypes: Quantity, Boolean, Enumerated, Text, Reference.

Quantity properties bind:

- `quantityKind` → frozen QuantityKind / ontology QuantityType (`@os20/physical#Torque`)
- `dimension` token (`M L^2 T^-2`)
- `unit` (`N.m` or `@os20/units-v1#…`)
- `origin` measured | calculated | simulated

Ontology does **not** convert units. Mismatch vs catalog dimension → `OS20-E4014`.

Force = M·L·T⁻², Velocity = L·T⁻¹, Torque = M·L²·T⁻² are declared on quantity types; `os20-quantities` checks algebra.
