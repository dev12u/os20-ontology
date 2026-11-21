//! Canonical semantic property IDs and SysML feature bindings.
//!
//! SysML attributes keep their local names (`mass`). Engineering identity uses
//! fully qualified feature paths plus optional `os20:property:…` semantic IDs.
//! Bare display names are never global identity.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::diagnostics::{DiagnosticCode, OntologyDiagnostic};

/// Canonical semantic property id (`os20:property:equipment.dryMass`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticPropertyId(String);

impl SemanticPropertyId {
    /// Validate and construct.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, String> {
        let raw = raw.as_ref().trim();
        let Some(rest) = raw.strip_prefix("os20:property:") else {
            return Err(format!(
                "semantic property id must start with `os20:property:`, got `{raw}`"
            ));
        };
        if rest.is_empty()
            || !rest
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
            || rest.starts_with('.')
            || rest.ends_with('.')
            || rest.contains("..")
        {
            return Err(format!("invalid semantic property local part in `{raw}`"));
        }
        Ok(Self(raw.to_owned()))
    }

    /// Borrow the canonical string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SemanticPropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SemanticPropertyId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Serialize for SemanticPropertyId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SemanticPropertyId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

/// One standardized engineering property.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPropertyDef {
    /// Canonical id.
    pub id: SemanticPropertyId,
    /// Human-readable display name (need not be globally unique).
    pub name: String,
    /// Engineering meaning.
    pub description: String,
    /// Physical quantity kind (`mass`, `length`, `pressure`, …).
    pub quantity_kind: String,
    /// Preferred unit when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_unit: Option<String>,
    /// Optional category applicability.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    /// Source ontology package (`@os20/core`).
    pub source_package: String,
    /// Source package version.
    pub source_version: String,
    /// Legacy portal taxonomy ids (`physical.mass`) for back-compat.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legacy_semantic_ids: Vec<String>,
}

/// Explicit SysML feature → semantic property binding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPropertyBinding {
    /// Fully qualified SysML feature (`Pkg::Def::mass` or `@pkg#Owner.mass`).
    pub sysml_feature: String,
    /// Canonical semantic id.
    pub semantic_id: SemanticPropertyId,
}

/// Display payload for UI / catalog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticPropertyDisplay {
    /// Display name.
    pub display_name: String,
    /// Semantic meaning (description).
    pub semantic_meaning: String,
    /// Quantity kind.
    pub quantity: String,
    /// Canonical unit, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Canonical semantic id.
    pub semantic_id: String,
    /// Source ontology package.
    pub source_package: String,
    /// Bound SysML feature, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysml_feature: Option<String>,
}

/// Resolved property after FQN + binding lookup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSemanticProperty {
    /// Qualified SysML feature that was resolved.
    pub sysml_feature: String,
    /// Bound semantic definition.
    pub definition: SemanticPropertyDef,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PropertiesFile {
    #[serde(default)]
    properties: Vec<SemanticPropertyDef>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BindingsFile {
    #[serde(default)]
    bindings: Vec<SemanticPropertyBinding>,
}

/// In-memory registry of semantic property defs + bindings.
#[derive(Clone, Debug, Default)]
pub struct SemanticPropertyRegistry {
    defs: BTreeMap<SemanticPropertyId, SemanticPropertyDef>,
    /// Normalized feature path → semantic id.
    bindings: BTreeMap<String, SemanticPropertyId>,
    /// Diagnostics accumulated during load/register.
    diagnostics: Vec<OntologyDiagnostic>,
}

impl SemanticPropertyRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load property definitions from JSON (`os20.semantic-properties/v1`).
    pub fn load_properties_json(&mut self, json: &str) -> Result<(), String> {
        let file: PropertiesFile =
            serde_json::from_str(json).map_err(|e| format!("semantic properties JSON: {e}"))?;
        for def in file.properties {
            self.register_def(def);
        }
        Ok(())
    }

    /// Load feature bindings from JSON (`os20.semantic-property-bindings/v1`).
    pub fn load_bindings_json(&mut self, json: &str) -> Result<(), String> {
        let file: BindingsFile =
            serde_json::from_str(json).map_err(|e| format!("semantic bindings JSON: {e}"))?;
        for b in file.bindings {
            self.bind(b.sysml_feature, b.semantic_id);
        }
        Ok(())
    }

    /// Register a definition. Conflicting metadata for the same id → E4029.
    pub fn register_def(&mut self, def: SemanticPropertyDef) {
        if let Some(existing) = self.defs.get(&def.id) {
            if existing.quantity_kind != def.quantity_kind
                || existing.canonical_unit != def.canonical_unit
                || existing.name != def.name
                || existing.description != def.description
            {
                self.diagnostics.push(OntologyDiagnostic::new(
                    DiagnosticCode::ConflictingSemanticProperty,
                    format!(
                        "conflicting semantic property definition for `{}`",
                        def.id.as_str()
                    ),
                ));
                return;
            }
            // Identical — idempotent.
            return;
        }
        self.defs.insert(def.id.clone(), def);
    }

    /// Bind a fully qualified SysML feature to a semantic id.
    pub fn bind(&mut self, sysml_feature: impl Into<String>, semantic_id: SemanticPropertyId) {
        let feature = normalize_feature_key(&sysml_feature.into());
        if !self.defs.contains_key(&semantic_id) {
            self.diagnostics.push(
                OntologyDiagnostic::new(
                    DiagnosticCode::DanglingSemanticId,
                    format!(
                        "binding for `{feature}` references unknown semantic id `{}`",
                        semantic_id.as_str()
                    ),
                ),
            );
            return;
        }
        if let Some(prev) = self.bindings.get(&feature) {
            if prev != &semantic_id {
                self.diagnostics.push(OntologyDiagnostic::new(
                    DiagnosticCode::ConflictingSemanticProperty,
                    format!(
                        "feature `{feature}` already bound to `{}`, cannot also bind `{}`",
                        prev.as_str(),
                        semantic_id.as_str()
                    ),
                ));
                return;
            }
        }
        self.bindings.insert(feature, semantic_id);
    }

    /// Resolve by fully qualified SysML feature.
    pub fn resolve_feature(&self, sysml_feature: &str) -> Option<ResolvedSemanticProperty> {
        let key = normalize_feature_key(sysml_feature);
        let id = self.bindings.get(&key)?;
        let definition = self.defs.get(id)?.clone();
        Some(ResolvedSemanticProperty {
            sysml_feature: key,
            definition,
        })
    }

    /// Lookup definition by canonical id.
    pub fn definition(&self, id: &SemanticPropertyId) -> Option<&SemanticPropertyDef> {
        self.defs.get(id)
    }

    /// Display metadata for a bound feature (or bare semantic id).
    pub fn display_for_feature(&self, sysml_feature: &str) -> Option<SemanticPropertyDisplay> {
        let resolved = self.resolve_feature(sysml_feature)?;
        Some(display_from_def(
            &resolved.definition,
            Some(resolved.sysml_feature),
        ))
    }

    /// Display metadata for a semantic id.
    pub fn display_for_id(&self, id: &SemanticPropertyId) -> Option<SemanticPropertyDisplay> {
        self.defs.get(id).map(|d| display_from_def(d, None))
    }

    /// Resolve an unqualified local name against candidate FQNs.
    ///
    /// - One distinct bound semantic id among candidates → Ok(Some)
    /// - Multiple distinct semantic ids → E4028
    /// - No bindings → W4002 and Ok(None)
    pub fn resolve_by_local_name(
        &self,
        local: &str,
        candidates: &[String],
    ) -> Result<Option<ResolvedSemanticProperty>, OntologyDiagnostic> {
        let local = local.trim();
        let mut matched: Vec<ResolvedSemanticProperty> = Vec::new();
        let mut seen_ids: BTreeSet<SemanticPropertyId> = BTreeSet::new();
        for c in candidates {
            let key = normalize_feature_key(c);
            if feature_local_name(&key) != local {
                continue;
            }
            if let Some(r) = self.resolve_feature(&key) {
                if seen_ids.insert(r.definition.id.clone()) {
                    matched.push(r);
                }
            }
        }
        match matched.len() {
            0 => Err(OntologyDiagnostic::new(
                DiagnosticCode::AmbiguousLegacyPropertyName,
                format!(
                    "unbound legacy property name `{local}` matched {} candidate(s) without semantic binding",
                    candidates
                        .iter()
                        .filter(|c| feature_local_name(&normalize_feature_key(c)) == local)
                        .count()
                ),
            )),
            1 => Ok(Some(matched.remove(0))),
            _ => Err(OntologyDiagnostic::new(
                DiagnosticCode::AmbiguousPropertyReference,
                format!(
                    "unqualified property `{local}` matches multiple semantic ids: {}",
                    matched
                        .iter()
                        .map(|m| m.definition.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )),
        }
    }

    /// Verify an observed quantity kind against the semantic definition.
    pub fn check_quantity(
        &self,
        semantic_id: &SemanticPropertyId,
        observed_quantity_kind: &str,
    ) -> Result<(), OntologyDiagnostic> {
        let Some(def) = self.defs.get(semantic_id) else {
            return Err(OntologyDiagnostic::new(
                DiagnosticCode::DanglingSemanticId,
                format!("unknown semantic property `{}`", semantic_id.as_str()),
            ));
        };
        if quantity_kinds_compatible(&def.quantity_kind, observed_quantity_kind) {
            Ok(())
        } else {
            Err(OntologyDiagnostic::new(
                DiagnosticCode::SemanticQuantityMismatch,
                format!(
                    "property `{}` expects quantity `{}` but observed `{}`",
                    semantic_id.as_str(),
                    def.quantity_kind,
                    observed_quantity_kind
                ),
            ))
        }
    }

    /// Accumulated load/register diagnostics.
    pub fn diagnostics(&self) -> &[OntologyDiagnostic] {
        &self.diagnostics
    }

    /// All definitions.
    pub fn definitions(&self) -> impl Iterator<Item = &SemanticPropertyDef> {
        self.defs.values()
    }

    /// All bindings (normalized feature → id).
    pub fn bindings(&self) -> impl Iterator<Item = (&str, &SemanticPropertyId)> {
        self.bindings.iter().map(|(k, v)| (k.as_str(), v))
    }
}

fn display_from_def(def: &SemanticPropertyDef, feature: Option<String>) -> SemanticPropertyDisplay {
    SemanticPropertyDisplay {
        display_name: def.name.clone(),
        semantic_meaning: def.description.clone(),
        quantity: def.quantity_kind.clone(),
        unit: def.canonical_unit.clone(),
        semantic_id: def.id.as_str().to_owned(),
        source_package: def.source_package.clone(),
        sysml_feature: feature,
    }
}

/// Normalize feature keys so `Pkg::Def::mass`, `Pkg::Def.mass`, and
/// `@pkg#Def.mass` compare consistently where possible.
pub fn normalize_feature_key(raw: &str) -> String {
    let s = raw.trim();
    if let Some((pkg, local)) = s.split_once('#') {
        // `@pkg#Owner.feature` — keep owner-qualified ElementId form.
        return format!("{}#{}", pkg.trim(), local.trim());
    }
    // `A::B::mass` stays; `A::B.mass` → `A::B::mass`.
    let parts: Vec<&str> = s
        .split("::")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return s.to_owned();
    }
    let mut out: Vec<&str> = Vec::new();
    let last = parts.len() - 1;
    for (i, p) in parts.iter().enumerate() {
        if i == last && p.contains('.') {
            out.extend(p.split('.').filter(|seg| !seg.is_empty()));
        } else {
            out.push(p);
        }
    }
    out.join("::")
}

fn feature_local_name(normalized: &str) -> &str {
    if let Some((_, local)) = normalized.split_once('#') {
        return local.rsplit_once('.').map(|(_, f)| f).unwrap_or(local);
    }
    normalized
        .rsplit_once("::")
        .map(|(_, f)| f)
        .unwrap_or(normalized)
}

/// Quantity kind compatibility (same engineering kind).
pub fn quantity_kinds_compatible(expected: &str, observed: &str) -> bool {
    normalize_quantity_kind(expected) == normalize_quantity_kind(observed)
}

fn normalize_quantity_kind(k: &str) -> String {
    let lower = k.trim().to_ascii_lowercase().replace('-', "").replace('_', "");
    match lower.as_str() {
        "massflow" | "massflowrate" => "massflowrate".to_owned(),
        "volumeflow" | "volumeflowrate" => "volumeflowrate".to_owned(),
        other => other.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_registry() -> SemanticPropertyRegistry {
        let mut r = SemanticPropertyRegistry::new();
        r.load_properties_json(
            r#"{
              "properties": [
                {
                  "id": "os20:property:equipment.dryMass",
                  "name": "Mass",
                  "description": "Equipment dry mass",
                  "quantityKind": "mass",
                  "canonicalUnit": "Kilogram",
                  "categories": ["equipment"],
                  "sourcePackage": "@os20/core",
                  "sourceVersion": "0.1.0",
                  "legacySemanticIds": ["physical.mass"]
                },
                {
                  "id": "os20:property:shipment.grossMass",
                  "name": "Mass",
                  "description": "Shipment gross mass including packaging",
                  "quantityKind": "mass",
                  "canonicalUnit": "Kilogram",
                  "categories": ["shipment"],
                  "sourcePackage": "@os20/core",
                  "sourceVersion": "0.1.0"
                },
                {
                  "id": "os20:property:fluid.massFlowRate",
                  "name": "Mass flow rate",
                  "description": "Mass of fluid per unit time",
                  "quantityKind": "massFlowRate",
                  "canonicalUnit": "KilogramPerSecond",
                  "categories": ["fluid"],
                  "sourcePackage": "@os20/physics",
                  "sourceVersion": "0.1.0"
                }
              ]
            }"#,
        )
        .unwrap();
        r.load_bindings_json(
            r#"{
              "bindings": [
                {
                  "sysmlFeature": "OS20_HVAC::Compressor::mass",
                  "semanticId": "os20:property:equipment.dryMass"
                },
                {
                  "sysmlFeature": "OS20_HVAC::Fan::mass",
                  "semanticId": "os20:property:equipment.dryMass"
                },
                {
                  "sysmlFeature": "Logistics::Crate::mass",
                  "semanticId": "os20:property:shipment.grossMass"
                }
              ]
            }"#,
        )
        .unwrap();
        r
    }

    #[test]
    fn two_owners_share_dry_mass_meaning() {
        let r = sample_registry();
        let a = r.resolve_feature("OS20_HVAC::Compressor::mass").unwrap();
        let b = r.resolve_feature("OS20_HVAC::Fan::mass").unwrap();
        assert_eq!(a.definition.id, b.definition.id);
        assert_eq!(a.definition.id.as_str(), "os20:property:equipment.dryMass");
    }

    #[test]
    fn same_local_name_different_semantics() {
        let r = sample_registry();
        let dry = r.resolve_feature("OS20_HVAC::Compressor::mass").unwrap();
        let gross = r.resolve_feature("Logistics::Crate::mass").unwrap();
        assert_ne!(dry.definition.id, gross.definition.id);
    }

    #[test]
    fn quantity_compatible_and_rejected() {
        let r = sample_registry();
        let id = SemanticPropertyId::new("os20:property:equipment.dryMass").unwrap();
        assert!(r.check_quantity(&id, "mass").is_ok());
        assert!(r.check_quantity(&id, "Mass").is_ok());
        let err = r.check_quantity(&id, "length").unwrap_err();
        assert_eq!(err.code, DiagnosticCode::SemanticQuantityMismatch);
        let flow = SemanticPropertyId::new("os20:property:fluid.massFlowRate").unwrap();
        let err2 = r.check_quantity(&flow, "mass").unwrap_err();
        assert_eq!(err2.code, DiagnosticCode::SemanticQuantityMismatch);
    }

    #[test]
    fn ambiguous_local_name_errors() {
        let r = sample_registry();
        let candidates = vec![
            "OS20_HVAC::Compressor::mass".into(),
            "Logistics::Crate::mass".into(),
        ];
        let err = r.resolve_by_local_name("mass", &candidates).unwrap_err();
        assert_eq!(err.code, DiagnosticCode::AmbiguousPropertyReference);
    }

    #[test]
    fn unbound_legacy_name_warns() {
        let r = sample_registry();
        let candidates = vec!["Other::Widget::mass".into()];
        let err = r.resolve_by_local_name("mass", &candidates).unwrap_err();
        assert_eq!(err.code, DiagnosticCode::AmbiguousLegacyPropertyName);
        assert!(!err.code.is_error());
    }
}
