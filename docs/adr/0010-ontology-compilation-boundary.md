# ADR 0010 — Ontology compilation boundary

## Status

Accepted

## Decision

The compiler does not parse KerML/SysML and does not replace `os20-language`. It consumes the bound `SourceGraph` the frozen frontend would emit (checked in as `ontology.sourcegraph.json` in Git trees). Annotations the language cannot yet express are explicit metadata on that graph, not a second file syntax.
