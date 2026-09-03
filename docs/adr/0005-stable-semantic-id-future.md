# ADR 0005 — Optional stable semantic ids (future)

## Status

Accepted (document only)

## Decision

Do not invent new KerML syntax in Prompt 1. Named `ElementId` includes the local name, so rename changes identity. A future optional explicit stable semantic id (independent of name) may be added if language-level support exists. Until then, evolution uses `ReplacedBy` / `Supersedes` guidance.
