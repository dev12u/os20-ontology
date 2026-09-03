# ADR 0009 — Bootstrap boundary

## Status

Accepted

## Decision

Bootstrap IDs are protected. Only `@os20/core` / `@omg/kerml` may (re)declare their own IDs. Other packages extend via Specialize/Mixin/Extend/Specify. Accidental same-ID shadowing is `OS20-E4002` / `E4001`.
