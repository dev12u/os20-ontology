# Architecture

UIDS (semantic protocol) is **not a crate in this checkout**. The frozen OS20 identity stack is the substrate: named `@scope/name#Local` IDs, Specialize/Mixin/Extend, Git-backed ontology packages, `os20.lock`, rebuildable SQLite.

OS20 engineering ontology is **meaning**: System/Part/Geometry/Requirement/Simulation/… classified against that substrate.

```text
Git KerML/SysML  →  os20-language  →  BoundSourceGraph
                                          ↓
                                   OntologyCompiler
                                          ↓
                                   OntologyRuntime / sdk.ontology()
                                          ↓
                          EngineeringGraph (instances + authored edges)
                                          ↓
                     validate_engineering_graph  ·  query  ·  JSON-LD export
```

JSON-LD serializes the same IDs (`@id` = ElementId). It does not replace identity.

Structural containment is recursive `contains` (authored); `partOf` is derived. Subsystem/component are contextual roles, not extra root classes.
