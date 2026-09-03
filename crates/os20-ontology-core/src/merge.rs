//! Property merge: existing ranking, ontology type checks only.

use crate::diagnostics::OntologyDiagnostic;
use crate::identity::{PropertyId, TypeId};
use crate::model::EffectiveProperty;
use crate::package::TypeRef;
use crate::snapshot::OntologySnapshot;

/// Merged feature plus optional override diagnostic.
#[derive(Clone, Debug)]
pub struct MergedProperty {
    /// Effective property after ranking.
    pub property: EffectiveProperty,
    /// Incompatible covariant override, if any.
    pub diagnostic: Option<OntologyDiagnostic>,
}

impl OntologySnapshot {
    /// Merge declared features. Closer Specialize declarations outrank mixins
    /// of the same ancestor. Ontology does not change that ranking; it only
    /// checks redefinition assignability.
    pub fn merge_properties(&self, ty: &TypeId) -> Vec<MergedProperty> {
        let ranked = self.properties(ty);
        let mut out = Vec::new();
        for p in ranked {
            let diagnostic = match (&p.ty, self.inherited_type_for(ty, &p.id)) {
                (TypeRef::Bound { id: local }, Some(orig)) if local != &orig => {
                    self.redefinition_compatible(&orig, local).err()
                }
                _ => None,
            };
            out.push(MergedProperty {
                property: p,
                diagnostic,
            });
        }
        out
    }

    fn inherited_type_for(&self, ty: &TypeId, prop: &PropertyId) -> Option<TypeId> {
        let mut first = None;
        for anc in self.supertypes(ty).into_iter().skip(1) {
            if let Some(d) = self.type_decl(&anc) {
                for p in &d.properties {
                    if &p.id == prop {
                        if let TypeRef::Bound { id } = &p.ty {
                            first = Some(id.clone());
                        }
                    }
                }
            }
        }
        first
    }
}
