# Canonical relations

Specialize/Mixin/Extend/Compose stay on the identity substrate.

Authored (store these): `instanceOf`, `contains` (`@os20/core#contains`), `hasProperty`, `hasGeometry`, `hasBehavior`, `madeOf`, `hasMaterialAssignment`, `appliesToRegion`, `hasPort`, `connects`, `realizes`, `implements`, `requires`, `allocatedTo`, `refines`, `conflictsWith`, `verifiedBy`, `validatedBy`, `assertsSatisfaction` (intent only), `representedBy`, `describes`, `derivedFrom`, `simulates`, `usesModel`, `usesGeometry`, `usesMaterial`, `usesBoundaryCondition`, `usesSolver`, `producesResult`, `providesEvidenceFor`, `manufacturedBy`, `producedBy`, `executesOn`, `controls`, `compatibleWith`, `incompatibleWith`, `configures`, `selects`, `excludes`, `generatedBy`, `providedBy`.

Derived (do not author): `partOf`, `instantiates`, `connectedTo`, `hasAllocatedRequirement`, `simulatedBy`, `controlledBy`.

No `hasMotorGeometry` / `mechanicalOut` predicates. Direction lives on the port.

See `canonical_relations()` in `crates/os20-ontology-core/src/engineering/relations.rs`.
