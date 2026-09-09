# ADR 0013 — OS20 Engineering Ontology v1 Architecture

## Status

Accepted (engineering ontology v1)

## Context

UIDS is specified as the semantic protocol. This checkout does **not** contain a UIDS crate. The frozen OS20 identity protocol is used as that substrate:

- UID = `@scope/name#Local` (`ElementId` / `TypeId`)
- Specialize / Mixin / Extend / Compose owned by the identity/runtime
- Git packages, `os20.lock`, rebuildable SQLite
- JSON-LD is an export, not identity

OS20 engineering ontology is vocabulary **on top of** that substrate.

## Decision

1. **UIDS vs OS20.** Do not redesign identity, namespaces, specialization, package resolution, or storage. Engineering types live in `@os20/engineering`, `@os20/physical`, `@os20/software`, `@os20/assurance`, `@os20/analysis`, `@os20/lifecycle`.

2. **Semantic object ≠ artifact.** Geometry/behavior/results are engineering objects. Bytes are frozen `ArtifactDigest` via `representedBy`. STEP/SCXML/FMU are `format` tokens.

3. **Definition ≠ instance.** `instanceOf` targets `EngineeringDefinition`. Instances do not Specialize definitions as a substitute. Downstream packages instantiate exported definitions.

4. **Canonical relations.** ~40 relations. Authored: `contains`, `instanceOf`, `hasGeometry`, `hasPort`, `connects`, `allocatedTo`, … Derived: `partOf`, `instantiates`, `connectedTo`, `simulatedBy`, `controlledBy`. Do not redeclare `Specialize`.

5. **Simulation triple.** `SimulationDefinition` → `SimulationRun` (maps frozen `ExecutionRunId`) → `SimulationResult`. No second runner.

6. **Test triple.** `TestDefinition` ≠ `TestExecution` ≠ `TestResult`.

7. **Configuration ≠ specialization.** Specialization is subtype meaning. Configuration is a selected option set.

8. **Quantities.** Property nodes bind `quantityKind` / dimension / unit. Conversion and algebra stay in `os20-quantities`. Ontology emits `OS20-E4014` on dimension mismatch.

9. **Artifacts / provenance / validation.** Reuse `ArtifactDigest`, `ProvenanceDigest`, `RequirementId`, `ConstraintResult::{Satisfied,Violated,Indeterminate}`, `ValidationOutcome::{Pass,Fail,Inconclusive}`, `RequirementSatisfaction`. `assertsSatisfaction` is intent only.

10. **External standards.** Map/reuse (SysML, STEP, FMI, Modelica, SCXML, PROV-O, QUDT). Do not import those metamodels into core v1.

## Consequences

Product surfaces keep calling `sdk.ontology()`. Engineering graphs are a separate occurrence layer (`EngineeringGraph`) validated against the compiled snapshot. Unknown graph schema fails closed (`OS20-E4022`).
