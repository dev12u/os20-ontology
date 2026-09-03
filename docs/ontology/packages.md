# Ontology packages

Ontology packages are ordinary OS20 packages with `role = "ontology"` (or `kind = "ontology"`) in `os20.toml`. They use the existing resolver, `os20.lock` pins, and Git tree identity (`version`, `repository`, `commit`, `tree`, `package_root`, manifest digest). There is **no** ontology-specific lockfile or publish endpoint.

Example:

```toml
[package]
name = "@os20/core"
version = "0.1.0"
edition = "2026"
role = "ontology"
```

Dependencies are normal package dependencies (`@os20/physics`, `@os20/electrical`, …). Optional dependencies are honored only when the manifest already expresses them.

Git remains authority. KerML/SysML files live in the package; the frozen language frontend emits a bound `SourceGraph`. The ontology compiler never parses tokens.
