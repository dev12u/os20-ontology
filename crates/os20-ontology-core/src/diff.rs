//! Ontology diff + SemVer advice. Does not rewrite historical IDs.

use serde::{Deserialize, Serialize};

use crate::identity::{PredicateId, PropertyId, TypeId};
use crate::package::Visibility;
use crate::snapshot::OntologySnapshot;

/// Diff category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OntologyDiffKind {
    /// New type.
    TypeAdded,
    /// Removed type.
    TypeRemoved,
    /// Specialize edges changed.
    SpecializationChanged,
    /// Feature added.
    FeatureAdded,
    /// Feature removed.
    FeatureRemoved,
    /// Feature type changed.
    FeatureTypeChanged,
    /// Predicate changed.
    PredicateChanged,
    /// Domain membership changed.
    DomainMembershipChanged,
    /// Deprecated.
    Deprecation,
    /// Replacement mapping added/changed.
    ReplacementMapping,
}

/// One atomic change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyChange {
    /// Category.
    pub kind: OntologyDiffKind,
    /// Primary type.
    pub type_id: Option<TypeId>,
    /// Feature if applicable.
    pub property: Option<PropertyId>,
    /// Predicate if applicable.
    pub predicate: Option<PredicateId>,
    /// Public contract?
    pub public: bool,
}

/// SemVer advice (existing analyzer shape).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SemVerAdvice {
    /// Breaking public contract.
    Major,
    /// Compatible public addition.
    Minor,
    /// Non-contractual / deprecation metadata.
    Patch,
    /// No semantic change.
    None,
}

/// Diff between two snapshots (typically two ontology versions).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyDiff {
    /// Changes, deterministic order.
    pub changes: Vec<OntologyChange>,
    /// Advice.
    pub semver: SemVerAdvice,
}

/// Diff `old` → `new`. Old IDs are never rewritten to replacements.
pub fn ontology_diff(old: &OntologySnapshot, new: &OntologySnapshot) -> OntologyDiff {
    let mut changes = Vec::new();
    let old_ids: Vec<_> = old.type_ids().cloned().collect();
    let new_ids: Vec<_> = new.type_ids().cloned().collect();
    for id in &new_ids {
        if old.type_decl(id).is_none() {
            let public = old_or_new_public(new, id);
            changes.push(ch(OntologyDiffKind::TypeAdded, Some(id.clone()), public));
        }
    }
    for id in &old_ids {
        if new.type_decl(id).is_none() {
            let public = old_or_new_public(old, id);
            changes.push(ch(OntologyDiffKind::TypeRemoved, Some(id.clone()), public));
        }
    }
    for id in &old_ids {
        let Some(o) = old.type_decl(id) else { continue };
        let Some(n) = new.type_decl(id) else { continue };
        let public = n.visibility == Visibility::Public;
        if o.specializes != n.specializes {
            changes.push(ch(
                OntologyDiffKind::SpecializationChanged,
                Some(id.clone()),
                public,
            ));
        }
        if o.domains != n.domains {
            changes.push(ch(
                OntologyDiffKind::DomainMembershipChanged,
                Some(id.clone()),
                public,
            ));
        }
        if format!("{:?}", o.lifecycle) != format!("{:?}", n.lifecycle) {
            changes.push(ch(OntologyDiffKind::Deprecation, Some(id.clone()), public));
        }
        let op: Vec<_> = o.properties.iter().map(|p| p.id.clone()).collect();
        let np: Vec<_> = n.properties.iter().map(|p| p.id.clone()).collect();
        for pid in &np {
            if !op.contains(pid) {
                changes.push(OntologyChange {
                    kind: OntologyDiffKind::FeatureAdded,
                    type_id: Some(id.clone()),
                    property: Some(pid.clone()),
                    predicate: None,
                    public,
                });
            }
        }
        for pid in &op {
            if !np.contains(pid) {
                changes.push(OntologyChange {
                    kind: OntologyDiffKind::FeatureRemoved,
                    type_id: Some(id.clone()),
                    property: Some(pid.clone()),
                    predicate: None,
                    public,
                });
            } else {
                let ot = o.properties.iter().find(|p| &p.id == pid).map(|p| &p.ty);
                let nt = n.properties.iter().find(|p| &p.id == pid).map(|p| &p.ty);
                if ot != nt {
                    changes.push(OntologyChange {
                        kind: OntologyDiffKind::FeatureTypeChanged,
                        type_id: Some(id.clone()),
                        property: Some(pid.clone()),
                        predicate: None,
                        public,
                    });
                }
            }
        }
    }
    let old_preds: Vec<_> = old.predicates().map(|p| p.id.clone()).collect();
    let new_preds: Vec<_> = new.predicates().map(|p| p.id.clone()).collect();
    if old_preds != new_preds {
        changes.push(OntologyChange {
            kind: OntologyDiffKind::PredicateChanged,
            type_id: None,
            property: None,
            predicate: new_preds.first().cloned(),
            public: true,
        });
    }
    let old_ev: Vec<_> = old.evolution().iter().map(|e| format!("{:?}", e)).collect();
    let new_ev: Vec<_> = new.evolution().iter().map(|e| format!("{:?}", e)).collect();
    if old_ev != new_ev {
        changes.push(ch(OntologyDiffKind::ReplacementMapping, None, true));
    }
    changes.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
    let semver = advise(&changes);
    OntologyDiff { changes, semver }
}

fn ch(kind: OntologyDiffKind, type_id: Option<TypeId>, public: bool) -> OntologyChange {
    OntologyChange {
        kind,
        type_id,
        property: None,
        predicate: None,
        public,
    }
}

fn old_or_new_public(snap: &OntologySnapshot, id: &TypeId) -> bool {
    snap.type_decl(id)
        .map(|d| d.visibility == Visibility::Public)
        .unwrap_or(true)
}

fn advise(changes: &[OntologyChange]) -> SemVerAdvice {
    let mut major = false;
    let mut minor = false;
    let mut patch = false;
    for c in changes {
        if !c.public {
            continue;
        }
        match c.kind {
            OntologyDiffKind::TypeRemoved
            | OntologyDiffKind::SpecializationChanged
            | OntologyDiffKind::FeatureRemoved
            | OntologyDiffKind::FeatureTypeChanged => major = true,
            OntologyDiffKind::TypeAdded | OntologyDiffKind::FeatureAdded => minor = true,
            OntologyDiffKind::Deprecation
            | OntologyDiffKind::ReplacementMapping
            | OntologyDiffKind::DomainMembershipChanged
            | OntologyDiffKind::PredicateChanged => patch = true,
        }
    }
    if major {
        SemVerAdvice::Major
    } else if minor {
        SemVerAdvice::Minor
    } else if patch {
        SemVerAdvice::Patch
    } else {
        SemVerAdvice::None
    }
}
