# OS20 engineering ontology v1

Engineering meaning on the frozen OS20 identity substrate (UIDS-equivalent in this checkout: `@scope/name#Local`, Specialize/Mixin, Git packages).

```text
UIDS / OS20 identity protocol
        ↓
OS20 engineering vocabulary  (@os20/engineering …)
        ↓
EngineeringGraph (occurrences)
        ↓
sdk.ontology() + validate_engineering_graph + query
```

- Spec freeze of the *compiler/runtime*: still [ontology-spec-v1.md](../ontology-spec-v1.md)
- Engineering v1 ADR: [0013](../adr/0013-os20-engineering-ontology-v1.md)
- Packages: `packages/`
- Implementation: `crates/os20-ontology-core/src/engineering/`

Index:

| Doc | Topic |
| --- | --- |
| [architecture.md](architecture.md) | Layering, UIDS boundary |
| [core-model.md](core-model.md) | EngineeringThing layers |
| [definition-instance.md](definition-instance.md) | instanceOf |
| [relations.md](relations.md) | Canonical predicates |
| [geometry.md](geometry.md) | Geometry vs CAD formats |
| [properties.md](properties.md) | QuantityProperty |
| [interfaces.md](interfaces.md) | Ports / connections |
| [requirements-validation.md](requirements-validation.md) | Frozen outcomes |
| [simulation.md](simulation.md) | Def / run / result |
| [artifacts.md](artifacts.md) | representedBy |
| [external-mappings.md](external-mappings.md) | SysML, STEP, FMI, … |
| [versioning.md](versioning.md) | Schema 1 fail-closed |
| [v1-scope.md](v1-scope.md) | In / deferred |
