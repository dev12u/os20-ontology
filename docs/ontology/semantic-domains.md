# Semantic domains

A **semantic domain** is an ontology-defined, versioned concept (`SemanticDomainId`), not a package and not a closed Rust enum.

Baseline domains defined by `@os20/core`: Design, Structure, Behaviour, Requirements, Verification, Validation, Simulation, Geometry, Manufacturing, Physics, Safety, Compliance, plus Electrical as used by the core catalog.

A type may belong to **many** domains. A package may define **many** domains. Do not assume one package ≡ one domain.

Quantity **dimensions** (`M`, `L`, `T`, …) are physical-dimension tokens on `QuantityType`, not semantic domains.
