# OS20 ontology documentation

OS20 ontology is the **semantic vocabulary of Systems as Code**.

- Language: KerML / SysML  
- History: Git  
- Distribution: ordinary OS20 packages (`PackageRole::Ontology`)  
- Interpretation: `OntologyRuntime` in `os20-sdk`  
- Application to models: SemanticResolver  
- Acceleration: rebuildable SQLite / `.os20`  
- Release metadata: existing registry  
- Exposure: CLI, LSP, Workbench — none of which redefine meaning  

Canonical contract: [ontology-spec-v1.md](../ontology-spec-v1.md)

Engineering domain vocabulary (v1): [engineering/README.md](../engineering/README.md)

| Topic | Doc |
| --- | --- |
| Architecture | [architecture.md](architecture.md) |
| Identity | [identity.md](identity.md) |
| Packages | [packages.md](packages.md) |
| Compiler | [compiler.md](compiler.md) |
| Runtime | [runtime.md](runtime.md) |
| Type system | [type-system.md](type-system.md) |
| Predicates | [predicates.md](predicates.md) |
| Domains | [semantic-domains.md](semantic-domains.md) |
| Physics / quantities | [physics-foundation.md](physics-foundation.md) |
| Evolution | [evolution.md](evolution.md) |
| Migration | [migration.md](migration.md) |
| Versioning | [versioning.md](versioning.md) |
| Offline | [offline.md](offline.md) |
| Product usage | [product.md](product.md) / [product-usage.md](product-usage.md) |
| Conformance | [conformance.md](conformance.md) |
| Bootstrap | [bootstrap.md](bootstrap.md) |
| Fingerprints | [fingerprints.md](fingerprints.md) |
| Freeze | [freeze.md](freeze.md) |
