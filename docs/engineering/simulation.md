# Simulation

Keep separate: Definition, Model, Scenario, Run, Input/Output/Parameter, Result, SolverReference, Environment, Boundary/Initial condition.

`SimulationRun.executionRun` holds frozen `ExecutionRunId`. Execution is `ExecutionDefinition → ExecutionPlanDigest → ExecutionRunId → ArtifactDigest + ProvenanceDigest`.

Result ≠ file: `TemperatureFieldResult representedBy ArtifactDigest`.

Disciplines in v1: StructuralSimulation, ThermalSimulation, SystemSimulation. FEA/CFD are method tokens, not classes.

Authored: `simulates`, `usesModel`, `usesGeometry`, `usesMaterial`, `usesBoundaryCondition`, `usesSolver`, `producesResult`. Derived: `simulatedBy`.
