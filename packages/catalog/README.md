# Semantic property catalog (sidecar)

Canonical engineering property identities live here as JSON sidecars — not as a
new database or service.

| File | Role |
|------|------|
| `semantic-properties.json` | `os20:property:…` definitions (name, description, quantity, unit, source) |
| `semantic-property-bindings.json` | Fully qualified SysML features → semantic ids |

**Rules**

- SysML attributes keep their local names (`mass`). Do not rename them for identity.
- Bare names are never global identity. Use owner-qualified `PropertyId`
  (`@pkg#Owner.mass`) and/or explicit bindings.
- Two owners may share one semantic id (both equipment dry mass) or bind the same
  local name to different ids (dry mass vs gross mass).
- Legacy portal ids (`physical.mass`) remain valid via `legacySemanticIds`.
