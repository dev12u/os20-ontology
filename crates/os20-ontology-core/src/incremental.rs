//! Incremental ontology projection: SourceGraphDelta → affected elements.

use std::collections::BTreeSet;

use crate::fingerprint::ContentDigest;
use crate::identity::TypeId;
use crate::snapshot::OntologySnapshot;
use crate::source_graph::SourceGraphDelta;

/// What must be recomputed after a delta.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InvalidationReport {
    /// Types whose semantic payload changed.
    pub types: BTreeSet<TypeId>,
    /// Compatibility/effective-property queries to drop.
    pub compatibility: BTreeSet<TypeId>,
    /// Semantic fingerprint unchanged (comment-only / file move).
    pub semantic_noop: bool,
}

/// Compute invalidation between two snapshots and a language delta.
pub fn invalidate_from_delta(
    before: &OntologySnapshot,
    after: &OntologySnapshot,
    delta: &SourceGraphDelta,
) -> InvalidationReport {
    if delta.comment_only && before.semantic_fingerprint() == after.semantic_fingerprint() {
        return InvalidationReport {
            semantic_noop: true,
            ..InvalidationReport::default()
        };
    }
    let mut types = BTreeSet::new();
    let mut compatibility = BTreeSet::new();
    let before_ids: BTreeSet<_> = before.type_ids().cloned().collect();
    let after_ids: BTreeSet<_> = after.type_ids().cloned().collect();
    for id in before_ids.union(&after_ids) {
        let b = before.type_decl(id).map(|t| t.semantic_canonical());
        let a = after.type_decl(id).map(|t| t.semantic_canonical());
        if b != a {
            types.insert(id.clone());
            compatibility.insert(id.clone());
            for sub in after.subtypes(id).into_iter().chain(before.subtypes(id)) {
                compatibility.insert(sub);
            }
        }
    }
    InvalidationReport {
        types,
        compatibility,
        semantic_noop: false,
    }
}

/// True when only comments differ (same semantic digest).
pub fn comment_only_edit(before: &ContentDigest, after: &ContentDigest) -> bool {
    before == after
}
