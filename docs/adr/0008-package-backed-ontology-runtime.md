# ADR 0008 — Package-backed ontology runtime

## Status

Accepted (Prompt 2)

## Decision

Production ontology is compiled from locked OS20 ontology packages into `OntologySnapshot` (`OntologySourceKind::PackageBacked`). `MemoryOntology::core()` remains bootstrap/emergency only.
