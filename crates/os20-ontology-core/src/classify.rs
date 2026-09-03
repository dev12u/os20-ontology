//! Classification from Specialize ancestry (never from names).

use serde::{Deserialize, Serialize};

use crate::identity::{SemanticDomainId, TypeId};
use crate::snapshot::OntologySnapshot;

/// Root / physical / domain classification of a type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Classification {
    /// Type.
    pub type_id: TypeId,
    /// Specialize ancestors including self, toward `Thing` when present.
    pub ancestors: Vec<TypeId>,
    /// Specialize descendants excluding self.
    pub descendants: Vec<TypeId>,
    /// Engineering root (`Thing`) if this type specializes it.
    pub root: Option<TypeId>,
    /// True iff the type specializes `@os20/core#PhysicalThing`.
    pub is_physical: bool,
    /// True iff the type specializes `UnallocatedThing` or matches Unallocated{T}.
    pub is_unallocated: bool,
    /// Semantic domains (union along ancestry + declared).
    pub semantic_domains: Vec<SemanticDomainId>,
}

impl OntologySnapshot {
    /// Classify by ancestry and declared domains.
    pub fn classify(&self, id: &TypeId) -> Option<Classification> {
        let _ = self.type_decl(id)?;
        let ancestors = self.supertypes(id);
        let thing = TypeId::thing();
        let physical = TypeId::new("@os20/core#PhysicalThing").ok();
        let unalloc = TypeId::new("@os20/core#UnallocatedThing").ok();
        let is_physical = physical.as_ref().is_some_and(|p| self.is_subtype_of(id, p));
        let is_unallocated = unalloc.as_ref().is_some_and(|u| self.is_subtype_of(id, u));
        let mut semantic_domains = self.domains_for_element(id);
        semantic_domains.sort();
        semantic_domains.dedup();
        Some(Classification {
            type_id: id.clone(),
            descendants: self.subtypes(id),
            root: ancestors.iter().find(|t| *t == &thing).cloned(),
            ancestors,
            is_physical,
            is_unallocated,
            semantic_domains,
        })
    }

    /// Physicality from ancestry, not the substring "Physical".
    pub fn is_physical(&self, id: &TypeId) -> bool {
        self.classify(id).is_some_and(|c| c.is_physical)
    }

    /// Domains declared on the type and its Specialize ancestors.
    pub fn domains_for_element(&self, id: &TypeId) -> Vec<SemanticDomainId> {
        let mut out = Vec::new();
        for anc in self.supertypes(id) {
            if let Some(d) = self.type_decl(&anc) {
                out.extend(d.domains.iter().cloned());
            }
        }
        out.sort();
        out.dedup();
        out
    }

    /// Types that participate in `domain` (declared or inherited).
    pub fn elements_in_domain(&self, domain: &SemanticDomainId) -> Vec<TypeId> {
        self.type_ids()
            .filter(|id| self.domains_for_element(id).iter().any(|d| d == domain))
            .cloned()
            .collect()
    }
}
