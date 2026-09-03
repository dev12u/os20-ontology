# ADR 0007 — Refactoring and evolution relations

## Status

Accepted

## Decision

`Supersedes`, `ReplacedBy`, `SplitInto`, and `MergedInto` are evolution edges, not Specialize. `Unallocated<T>` is a classifier naming convention (`Unallocated{T}` specializing `T` and `UnallocatedThing`), not a keyword and not null.
