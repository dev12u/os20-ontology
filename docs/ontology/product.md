# Ontology product surface

Ontology **meaning** lives in `os20-sdk` (`sdk.ontology()`). CLI, LSP, and the Engineering Workbench only select, envelope, and render reports.

## SDK

Frozen handle: `Os20Sdk::ontology()`.

Operations: `packages`, `snapshot`, `fingerprint`, `resolve_type`, `find_types`, `type_summary`, `supertypes`, `subtypes`, `properties`, `predicates`, `domains`, `references`, `why_type`, `diff`, `impact`, plus product reports (`status_report`, `type_report`, `search_report`, `diff_report`, `impact_report`, `compatibility_report`).

Reports use schema 1 (`to_json_report`). No rusqlite, parser, or git2 types. No database row ids.

## CLI

`clap → sdk.ontology() → report → renderer`.

```text
os20 ontology status | list | show <type> | find <query>
os20 ontology parents <type> | children <type> | why <type> | references <type>
os20 ontology diff --from … --to …
os20 ontology impact <type>
```

JSON kinds: `ontology_status`, `ontology_type`, `ontology_search`, `ontology_diff`, `ontology_impact`.

All of these are read-only and work `--offline` against the locked SDK snapshot. The CLI does not parse ontology files, traverse graphs, open SQLite, talk to Git, call the registry, or compute type compatibility.

## LSP

Additive schema-1 methods (each handler is `sdk.ontology()` only):

`os20/ontologyStatus`, `ontologyType`, `ontologyFind`, `ontologyParents`, `ontologyChildren`, `ontologyReferences`, `ontologyWhy`, `ontologyDiff`, `ontologyImpact`, plus optional `ontologyCompatibility`.

Standard hover may append ontology type/package lines when `sdk.ontology().type_summary` already resolves the language element id. Completion and textual references stay on the language service. Go-to-definition stays on language locations (`file://` or `os20://`). Semantic ontology references use `os20/ontologyReferences`.

Locked ontology source URIs use the frozen `os20://source/v1/...` scheme.

## Workbench

View **OS20 Ontology** under the OS20 container. Per workspace: Ontology Packages, Semantic Domains, Types, Deprecated. Type nodes expand SDK-reported sections (supertypes, subtypes, properties, predicates, domains, references, evolution). Unallocated classifiers render as real types.

Commands: **Why This Type?**, **Find Ontology Type**, **Inspect Ontology Type**, **Check Type Compatibility**, **Compare Ontology Versions**. Graph filters reuse the existing semantic graph (`Specialize` / predicate / domain relation kinds). Diff/impact reuse existing explorers. No Workbench semantic cache; views hold response trees only and mark stale when the ontology fingerprint changes. No AuthZ UI.

Offline ontology source availability is whatever `sdk.workspace()` / readiness already reports. Each workspace folder has its own snapshot; two roots may show different package versions.
