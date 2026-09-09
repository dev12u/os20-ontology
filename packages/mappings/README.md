# External standard mappings (map/reuse, do not duplicate)

OS20 engineering ontology does **not** import foreign metamodels as core types.

| Standard | OS20 treatment |
| --- | --- |
| KerML / SysML v2 | Language frontend → bound SourceGraph. PartDefinition ↔ Physical/System definition; Port/Interface ↔ PortDefinition; Requirement ↔ frozen RequirementId + RequirementDefinition classification. |
| STEP | Geometry/product **artifact**. Adapter emits Geometry + representedBy ArtifactDigest. No EXPRESS schema in core. |
| FMI / FMU | Artifact `format=fmu`. Adapter: SimulationModel, ports, parameters. |
| Modelica | Map models/connectors/equations to SimulationModel / Port / QuantityProperty / frozen Constraint. Ontology does not execute Modelica. |
| SCXML | BehaviorArtifact `format=scxml` representing a StateMachine. |
| RDF / OWL / JSON-LD | Serialization of `@scope/name#Local` identities. Not a second UID. |
| PROV-O | `derivedFrom` ↔ wasDerivedFrom; `generatedBy` ↔ wasGeneratedBy. Frozen ProvenanceDigest remains authority. |
| QUDT | Mapping identities only. Dimension/Unit/QuantityKind stay in `os20-quantities`. |
