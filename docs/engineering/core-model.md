# Core model

Layered under `@os20/core#Thing` (not a flat noun list):

```text
EngineeringThing
 ├── EngineeringDefinition     reusable exported semantics
 ├── EngineeringInstance       occurrence; instanceOf → Definition
 ├── EngineeringProperty       Quantity/Boolean/Enum/Text/Reference
 ├── EngineeringModel          Geometry, SimulationModel, ThermalModel
 ├── EngineeringActivity       Verification, SimulationDefinition, manufacturing process
 ├── EngineeringResult         SimulationResult, TestResult, AnalysisResult
 ├── EngineeringEvidence       classification; identity is frozen Evidence/Artifact
 └── EngineeringDocument       document role; representedBy artifact
```

System / Physical / Assembly / Part / Device are definition+instance pairs under this layer. Equipment/Module are not v1 types. Subsystem/subassembly/component are `contains` roles.
