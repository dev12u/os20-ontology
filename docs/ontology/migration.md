# Ontology migration

`migration_analysis(old, new)` (also `sdk.ontology().migration_report`) produces a structured report:

- removed type IDs
- explicit replacements (`ReplacedBy` / `Supersedes`)
- splits (`SplitInto`)
- unresolved old feature-type references
- suggested target types from evolution guidance

`autoRewrite` is always **false**. Person → Individual | Organization | UnallocatedPerson does **not** silently re-parent existing children; they remain on their declared specialize edges until authors change them.

Historical locks keep historical snapshots. Publishing `@os20/physics@2` does not mutate a workspace still locked to `@os20/physics@1`.
