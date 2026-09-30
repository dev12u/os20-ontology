# Archived

This repository is archived. The live copy is in the runtime repository `os20`:

- production ontology crate: `os20/crates/os20-ontology`
- domain packages: `os20/ontologies/`
- ontology-track engine: `os20/ontology-track/` (not linked by the SDK façade)

Do not add a second production ontology implementation here.

# OS20 ontology

Semantic vocabulary for OS20 Systems as Code: compiler, runtime, type compatibility, predicates, domains, evolution, and SDK façade.

- Spec: [`docs/ontology-spec-v1.md`](docs/ontology-spec-v1.md)
- Docs: [`docs/ontology/README.md`](docs/ontology/README.md)

```text
git clone https://github.com/dev12u/os20-ontology.git
cd os20-ontology
cargo test --workspace
```

Requires a recent Rust toolchain (see `rust-toolchain.toml`).
