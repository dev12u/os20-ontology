//! Unallocated classifiers and reclassification analysis.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::diff::{SemVerAdvice, ontology_diff};
use crate::identity::TypeId;
use crate::snapshot::OntologySnapshot;

/// Reclassification: move children off an Unallocated\* classifier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReclassificationPlan {
    /// Placeholder.
    pub from: TypeId,
    /// New parent/subtype.
    pub to: TypeId,
    /// Direct children that remain valid classifiers.
    pub children: Vec<TypeId>,
    /// Downstream types affected.
    pub affected: Vec<TypeId>,
    /// SemVer consequence if this were a published ontology change.
    pub semver: SemVerAdvice,
}

/// Evolution guidance. Never silently rewrites IDs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionGuidance {
    /// Historical id (unchanged).
    pub id: TypeId,
    /// Deprecated?
    pub deprecated: bool,
    /// Replacement if declared (guidance only).
    pub replaced_by: Option<TypeId>,
    /// Split targets.
    pub split_into: Vec<TypeId>,
    /// Merged-into targets.
    pub merged_into: Vec<TypeId>,
}

impl OntologySnapshot {
    /// Unallocated classifiers are real types: they inherit and may have children.
    pub fn unallocated_children(&self, placeholder: &TypeId) -> Vec<TypeId> {
        self.direct_subtypes(placeholder)
    }

    /// Analyze moving children from `from` (typically Unallocated\*) to `to`.
    pub fn reclassify_analysis(&self, from: &TypeId, to: &TypeId) -> ReclassificationPlan {
        let children = self.unallocated_children(from);
        let mut affected = children.clone();
        affected.extend(self.subtypes(from));
        affected.sort();
        affected.dedup();
        ReclassificationPlan {
            from: from.clone(),
            to: to.clone(),
            children,
            affected,
            semver: SemVerAdvice::Major,
        }
    }

    /// Migration guidance without rewriting `id`.
    pub fn evolution_guidance(&self, id: &TypeId) -> EvolutionGuidance {
        use crate::evolution::EvolutionKind;
        use crate::package::LifecycleStatus;
        let deprecated = matches!(
            self.type_decl(id).map(|d| &d.lifecycle),
            Some(LifecycleStatus::Deprecated { .. })
        );
        let mut replaced_by = None;
        if let Some(LifecycleStatus::Deprecated { replacement }) =
            self.type_decl(id).map(|d| d.lifecycle.clone())
        {
            replaced_by = replacement.map(TypeId::from_element);
        }
        let mut split_into = Vec::new();
        let mut merged_into = Vec::new();
        for e in self.evolution() {
            if e.from.as_str() != id.as_str() {
                continue;
            }
            let to = TypeId::from_element(e.to.clone());
            match e.kind {
                EvolutionKind::ReplacedBy | EvolutionKind::Supersedes => {
                    replaced_by = Some(to);
                }
                EvolutionKind::SplitInto => split_into.push(to),
                EvolutionKind::MergedInto => merged_into.push(to),
            }
        }
        EvolutionGuidance {
            id: id.clone(),
            deprecated,
            replaced_by,
            split_into,
            merged_into,
        }
    }
}

/// Constraint description only — no general solver.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyConstraint {
    /// Constrained type.
    pub on: TypeId,
    /// Human/ontology description.
    pub description: String,
}

/// Existing semantic-diff reuse hook (ontology_diff).
pub fn reclassify_semver(before: &OntologySnapshot, after: &OntologySnapshot) -> SemVerAdvice {
    ontology_diff(before, after).semver
}

/// Structured migration analysis. Never rewrites historical IDs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    /// Semantic fingerprint of the older snapshot.
    pub from_fingerprint: String,
    /// Semantic fingerprint of the newer snapshot.
    pub to_fingerprint: String,
    /// Type ids present in `from` but not `to`.
    pub removed_ids: Vec<TypeId>,
    /// Explicit `ReplacedBy` / `Supersedes` pairs from the newer snapshot.
    pub replacements: Vec<(TypeId, TypeId)>,
    /// `SplitInto` groups (`from` → targets).
    pub splits: Vec<(TypeId, Vec<TypeId>)>,
    /// Old property/feature type refs whose target id is gone.
    pub unresolved_old_references: Vec<TypeId>,
    /// Suggested targets from evolution guidance (not applied).
    pub suggested_targets: Vec<(TypeId, Vec<TypeId>)>,
    /// Always false: products must not silent-rewrite models.
    pub auto_rewrite: bool,
}

/// Compare two ontology snapshots for human/tooling migration (no rewrite).
pub fn migration_analysis(old: &OntologySnapshot, new: &OntologySnapshot) -> MigrationReport {
    use crate::evolution::EvolutionKind;
    use crate::package::TypeRef;

    let old_ids: BTreeSet<_> = old.type_ids().cloned().collect();
    let new_ids: BTreeSet<_> = new.type_ids().cloned().collect();
    let mut removed_ids: Vec<TypeId> = old_ids.difference(&new_ids).cloned().collect();
    removed_ids.sort();

    let mut replacements = Vec::new();
    let mut split_map: BTreeMap<TypeId, Vec<TypeId>> = BTreeMap::new();
    for e in new.evolution() {
        let from = TypeId::from_element(e.from.clone());
        let to = TypeId::from_element(e.to.clone());
        match e.kind {
            EvolutionKind::ReplacedBy | EvolutionKind::Supersedes => {
                replacements.push((from, to));
            }
            EvolutionKind::SplitInto => {
                split_map.entry(from).or_default().push(to);
            }
            EvolutionKind::MergedInto => {}
        }
    }
    replacements.sort();
    replacements.dedup();
    let mut splits: Vec<(TypeId, Vec<TypeId>)> = split_map.into_iter().collect();
    for (_, v) in &mut splits {
        v.sort();
        v.dedup();
    }

    let mut unresolved_old_references = Vec::new();
    for t in old.type_ids() {
        if let Some(d) = old.type_decl(t) {
            for p in &d.properties {
                if let TypeRef::Bound { id } = &p.ty {
                    if !new_ids.contains(id) {
                        unresolved_old_references.push(id.clone());
                    }
                }
            }
        }
    }
    unresolved_old_references.sort();
    unresolved_old_references.dedup();

    let mut suggested_targets = Vec::new();
    for id in &removed_ids {
        let g = new.evolution_guidance(id);
        let mut targets = Vec::new();
        if let Some(r) = g.replaced_by {
            targets.push(r);
        }
        targets.extend(g.split_into);
        targets.extend(g.merged_into);
        targets.sort();
        targets.dedup();
        if !targets.is_empty() {
            suggested_targets.push((id.clone(), targets));
        }
    }

    MigrationReport {
        from_fingerprint: old.semantic_fingerprint().as_str().to_owned(),
        to_fingerprint: new.semantic_fingerprint().as_str().to_owned(),
        removed_ids,
        replacements,
        splits,
        unresolved_old_references,
        suggested_targets,
        auto_rewrite: false,
    }
}
