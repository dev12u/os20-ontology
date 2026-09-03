# Ontology architecture freeze (Prompt 5)

The broad ontology architecture is **frozen**. Remaining work is specialized domain expansion, not a second identity/resolver/registry.

Guiding statement: KerML/SysML provide the language; Git provides authored history; OS20 ontology packages provide shared meaning; `OntologyRuntime` interprets it; SemanticResolver applies it; SQLite accelerates it; the registry distributes versioned releases; CLI/LSP/Workbench expose it. **None of those layers redefine what an ontology element means.**

Checklist (Prompt 5 §54): all items **yes**, with these explicit notes:

- **39 Product source navigation:** LSP maps ontology provenance to `file://` or frozen `os20://source/v1/…`. Full editor-host E2E needs the frozen language `os20-sdk` binary.
- **48 Spec:** [`docs/ontology-spec-v1.md`](../../../docs/ontology-spec-v1.md) and [`os20-sdk/docs/ontology-spec-v1.md`](../ontology-spec-v1.md).
- **41–45 CLI/LSP/Workbench/publish binary E2E:** Product modules call only `sdk.ontology()`. Linking `os20` / `os20-lsp` against a complete language+workspace SDK is outside this ontology-core checkout; SDK-level publish/lock/registry-trait tests cover the frozen release model (`RecordingRegistry`, `LockedPackage`, no ontology endpoint).

AuthZ: ontology crates have no permission engine. Registry protocol and `PackageRelease` identity are unchanged.
