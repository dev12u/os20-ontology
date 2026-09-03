# ADR 0003 — Thing as engineering root

## Status

Accepted

## Decision

OS20 engineering classifiers ultimately specialize `Thing`. KerML language scalars remain `@omg/kerml` data types and are not renamed. No universal `Any` as an unresolved-type escape hatch. `Nothing` / missing / null / `Unallocated` stay distinct.
