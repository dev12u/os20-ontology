# Refactoring

`Unallocated{T}` is a real classifier: it specializes `T` and `UnallocatedThing`, can inherit, can own children, and can later be reclassified.

`reclassify_analysis` reports children, downstream types, and SemVer (Major for published public reparenting). It does not mutate historical packages.
