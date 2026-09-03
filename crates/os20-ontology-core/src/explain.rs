//! Explanations for type origin and assignability.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::identity::TypeId;
use crate::model::TypeSummary;
use crate::package::Provenance;
use crate::snapshot::OntologySnapshot;

/// Why a type exists in this snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhyType {
    /// Summary.
    pub summary: TypeSummary,
    /// Provenance.
    pub provenance: Provenance,
    /// Specialize path from the type to `Thing` when applicable.
    pub specialization_path: Vec<TypeId>,
    /// Packages that contributed mixins or ancestors.
    pub contributing_packages: Vec<crate::identity::PackageId>,
}

/// Why assignability holds or fails.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhyCompatible {
    /// Value type.
    pub from: TypeId,
    /// Target type.
    pub to: TypeId,
    /// Whether `from` is assignable to `to`.
    pub assignable: bool,
    /// Specialize path if assignable.
    pub path: Vec<TypeId>,
    /// Human explanation.
    pub reason: String,
}

/// Why a property has its effective type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhyPropertyType {
    /// Property.
    pub property: crate::identity::PropertyId,
    /// Owner queried.
    pub on: TypeId,
    /// Effective type.
    pub ty: Option<TypeId>,
    /// Declaring type.
    pub declared_by: Option<TypeId>,
    /// Ontology package version that supplied the declaring type.
    pub ontology_version: Option<String>,
    /// Explanation.
    pub reason: String,
}

/// Why an override was accepted or rejected.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhyOverride {
    /// Accepted?
    pub accepted: bool,
    /// Diagnostic token if rejected.
    pub diagnostic: Option<String>,
    /// Reason.
    pub reason: String,
}

/// Why an element is classified as PhysicalThing (or not).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhyClassified {
    /// Type.
    pub type_id: TypeId,
    /// Physical?
    pub is_physical: bool,
    /// Specialize path used.
    pub path: Vec<TypeId>,
    /// Ontology release that defined PhysicalThing.
    pub physical_thing_version: Option<String>,
    /// Reason.
    pub reason: String,
}

impl OntologySnapshot {
    /// Explain a type.
    pub fn why_type(&self, id: &TypeId) -> Option<WhyType> {
        let summary = self.type_summary(id)?;
        let path = self.specialize_path_to_thing(id);
        let mut pkgs: Vec<_> = path.iter().filter_map(|t| t.package().ok()).collect();
        pkgs.sort();
        pkgs.dedup();
        Some(WhyType {
            provenance: summary.provenance.clone(),
            summary,
            specialization_path: path,
            contributing_packages: pkgs,
        })
    }

    /// Specialize path `id → … → Thing` when Thing is an ancestor.
    pub fn specialize_path_to_thing(&self, id: &TypeId) -> Vec<TypeId> {
        let thing = TypeId::thing();
        self.specialize_path(id, &thing)
            .unwrap_or_else(|| self.supertypes(id))
    }

    /// One Specialize path from `from` to `to` (DAG; first BFS path).
    pub fn specialize_path(&self, from: &TypeId, to: &TypeId) -> Option<Vec<TypeId>> {
        if from == to {
            return Some(vec![from.clone()]);
        }
        let mut prev: BTreeMap<TypeId, TypeId> = BTreeMap::new();
        let mut q = VecDeque::new();
        q.push_back(from.clone());
        while let Some(cur) = q.pop_front() {
            for p in self
                .type_decl(&cur)
                .map(|d| d.specializes.clone())
                .unwrap_or_default()
            {
                if prev.contains_key(&p) || &p == from {
                    continue;
                }
                prev.insert(p.clone(), cur.clone());
                if &p == to {
                    let mut path = vec![to.clone()];
                    let mut at = to.clone();
                    while &at != from {
                        at = prev.get(&at)?.clone();
                        path.push(at.clone());
                    }
                    path.reverse();
                    return Some(path);
                }
                q.push_back(p);
            }
        }
        None
    }

    /// Explain assignability (`from` assignable to `to`).
    pub fn why_compatible(&self, from: &TypeId, to: &TypeId) -> WhyCompatible {
        let assignable = self.is_assignable_to(from, to);
        let path = if assignable {
            self.specialize_path(from, to).unwrap_or_default()
        } else {
            Vec::new()
        };
        let reason = match self.assignability(from, to) {
            crate::compatibility::Assignability::Same => "same type".into(),
            crate::compatibility::Assignability::Subtype => format!(
                "{} specializes {} (mixin ancestry is not used)",
                from.as_str(),
                to.as_str()
            ),
            crate::compatibility::Assignability::Rejected(r) => format!("{r:?}"),
        };
        WhyCompatible {
            from: from.clone(),
            to: to.clone(),
            assignable,
            path,
            reason,
        }
    }

    /// Why a property has this type.
    pub fn why_property_type(&self, on: &TypeId, name_suffix: &str) -> Option<WhyPropertyType> {
        let props = self.properties(on);
        let p = props
            .iter()
            .find(|p| p.id.as_str().ends_with(name_suffix))?;
        let ty = match &p.ty {
            crate::package::TypeRef::Bound { id } => Some(id.clone()),
            crate::package::TypeRef::Unresolved { .. } => None,
        };
        let ver = self
            .type_summary(&p.declared_by)
            .map(|s| s.ontology_version);
        Some(WhyPropertyType {
            property: p.id.clone(),
            on: on.clone(),
            ty,
            declared_by: Some(p.declared_by.clone()),
            ontology_version: ver,
            reason: format!(
                "property `{}` declared by `{}`",
                p.id.as_str(),
                p.declared_by.as_str()
            ),
        })
    }

    /// Why override accepted/rejected.
    pub fn why_override(&self, original: &TypeId, redefining: &TypeId) -> WhyOverride {
        match self.redefinition_compatible(original, redefining) {
            Ok(()) => WhyOverride {
                accepted: true,
                diagnostic: None,
                reason: "KerML covariant redefinition: redefining type specializes original".into(),
            },
            Err(d) => WhyOverride {
                accepted: false,
                diagnostic: Some(d.token),
                reason: d.message,
            },
        }
    }

    /// Why classified as PhysicalThing.
    pub fn why_classified_physical(&self, id: &TypeId) -> WhyClassified {
        let physical = TypeId::new("@os20/core#PhysicalThing").ok();
        let is_physical = physical.as_ref().is_some_and(|p| self.is_subtype_of(id, p));
        let path = physical
            .as_ref()
            .and_then(|p| self.specialize_path(id, p))
            .unwrap_or_default();
        let physical_thing_version = physical
            .as_ref()
            .and_then(|p| self.type_summary(p).map(|s| s.ontology_version));
        WhyClassified {
            type_id: id.clone(),
            is_physical,
            path,
            physical_thing_version,
            reason: if is_physical {
                "specializes @os20/core#PhysicalThing (not name matching)".into()
            } else {
                "does not specialize @os20/core#PhysicalThing".into()
            },
        }
    }
}
