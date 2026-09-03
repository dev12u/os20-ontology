//! Stable product reports for CLI/LSP/Workbench.
//!
//! These types are JSON-serializable SDK values. They must not expose
//! rusqlite, parser, or git2 implementation types, and they never carry
//! database row identifiers.

use serde::{Deserialize, Serialize};

use os20_ontology_core::{
    EvolutionGuidance, ImpactReport, OntologyDiff, OntologyPackage, OntologySnapshot,
    ReverseReferences, TypeId, TypeSummary, WhyCompatible, WhyType,
};

use crate::OntologyHandle;

/// JSON envelope schema. Additive fields are allowed; breaking changes need 2.
pub const REPORT_SCHEMA: u32 = 1;

/// Schema-1 envelope used by CLI JSON and as the inner body of LSP custom methods.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonEnvelope<T> {
    /// Schema version.
    pub schema: u32,
    /// Stable kind token.
    pub kind: String,
    /// Structured body (no implementation types).
    pub body: T,
}

/// Serialize a schema-1 report (CLI `--json`).
pub fn to_json_report<T: Serialize>(kind: &str, body: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(&JsonEnvelope {
        schema: REPORT_SCHEMA,
        kind: kind.to_owned(),
        body,
    })
}

/// Product-layer lookup failure (unknown selector / snapshot).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum OntologyQueryError {
    /// No type matched the selector.
    #[error("ontology type not found: {0}")]
    UnknownType(String),
    /// Selector matched more than one type.
    #[error("ontology type selector is ambiguous: {0}")]
    AmbiguousType(String),
    /// `--from` / `--to` snapshot is not loaded on this SDK instance.
    #[error("ontology snapshot not available: {0}")]
    UnknownSnapshot(String),
}

/// One ontology package as shown to products (lock/fingerprint health).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyPackageReport {
    /// `@scope/name`.
    pub package: String,
    /// Ontology release version.
    pub version: String,
    /// Source kind (`bootstrap`, `lockedPackages`, …).
    pub source: String,
    /// Git commit when the package is lock-backed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Package semantic fingerprint (hex).
    pub fingerprint: String,
}

impl OntologyPackageReport {
    fn from_package(pkg: &OntologyPackage, source: &str) -> Self {
        Self {
            package: pkg.ontology_id.to_string(),
            version: pkg.ontology_version.clone(),
            source: source.to_owned(),
            commit: pkg
                .source_revision
                .clone()
                .or_else(|| pkg.release.commit.clone()),
            fingerprint: pkg.fingerprint.as_str().to_owned(),
        }
    }
}

/// `ontology_status` / LSP `ontologyStatus`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyStatusReport {
    /// Packages in this workspace snapshot.
    pub packages: Vec<OntologyPackageReport>,
    /// Snapshot semantic fingerprint.
    pub fingerprint: String,
    /// Snapshot source fingerprint.
    pub source_fingerprint: String,
    /// Type count.
    pub type_count: usize,
    /// Domain count.
    pub domain_count: usize,
    /// Domain ids.
    pub domains: Vec<String>,
    /// Package count.
    pub package_count: usize,
    /// In-memory ontology runtime format (cache invalidation).
    pub runtime_format_version: u32,
    /// Compiled-in `@os20/core` bootstrap version (replaced by locked packages when present).
    pub core_ontology_version: String,
}

/// Expandable type explorer (no graph walk in products).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyTypeReport {
    /// Qualified type id.
    pub id: String,
    /// Summary (no DB ids).
    pub summary: TypeSummary,
    /// Direct parents.
    pub direct_parents: Vec<String>,
    /// Transitive supertypes including self.
    pub supertypes: Vec<String>,
    /// Transitive subtypes excluding self.
    pub subtypes: Vec<String>,
    /// Effective properties.
    pub properties: Vec<String>,
    /// Predicate ids whose endpoints mention this type.
    pub predicates: Vec<String>,
    /// Domain ids.
    pub domains: Vec<String>,
    /// Reverse type usages (owner, property).
    pub references: Vec<OntologyReferenceHit>,
    /// Deprecation / replacement / split / merge guidance (not a rewrite).
    pub evolution: EvolutionGuidance,
    /// True when this classifier is an Unallocated\* placeholder (still a real type).
    pub unallocated: bool,
    /// Declaring source file (package-relative).
    pub source_file: String,
    /// Declaring package.
    pub source_package: String,
    /// Source revision / commit when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
}

/// One reverse-reference hit.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyReferenceHit {
    /// Using type.
    pub owner: String,
    /// Property or predicate id.
    pub member: String,
}

/// `ontology_search` / LSP `ontologyFind`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologySearchReport {
    /// Query as received.
    pub query: String,
    /// Deterministic hits.
    pub types: Vec<TypeSummary>,
}

/// `ontology_diff` / LSP `ontologyDiff`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyDiffReport {
    /// From selector (opaque to products).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// To selector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Diff payload.
    pub diff: OntologyDiff,
}

/// `ontology_impact` / LSP `ontologyImpact`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyImpactReport {
    /// Origin type.
    pub origin: String,
    /// Impact payload.
    pub impact: ImpactReport,
}

impl<'a> OntologyHandle<'a> {
    /// Workspace ontology status (lock/fingerprint health for dashboards).
    pub fn status_report(&self) -> OntologyStatusReport {
        let snap = self.snapshot();
        let source = source_kind_token(snap);
        let mut packages: Vec<OntologyPackageReport> = snap
            .packages()
            .map(|p| OntologyPackageReport::from_package(p, source))
            .collect();
        packages.sort_by(|a, b| a.package.cmp(&b.package));
        OntologyStatusReport {
            fingerprint: snap.semantic_fingerprint().as_str().to_owned(),
            source_fingerprint: snap.source_fingerprint().as_str().to_owned(),
            type_count: snap.type_ids().count(),
            domain_count: snap.domains().count(),
            domains: snap.domains().map(|d| d.id.to_string()).collect(),
            package_count: packages.len(),
            packages,
            runtime_format_version: os20_ontology_core::ONTOLOGY_RUNTIME_FORMAT_VERSION,
            core_ontology_version: os20_ontology_core::OS20_CORE_ONTOLOGY_VERSION.to_owned(),
        }
    }

    /// Resolve a user/product selector via SDK search (not CLI graph walk).
    pub fn resolve_selector(&self, query: &str) -> Result<TypeId, OntologyQueryError> {
        let q = query.trim();
        if q.is_empty() {
            return Err(OntologyQueryError::UnknownType(q.to_owned()));
        }
        if let Ok(id) = TypeId::new(q) {
            if self.resolve_type(&id).is_some() {
                return Ok(id);
            }
        }
        let hits = self.find_types(q);
        match hits.len() {
            0 => Err(OntologyQueryError::UnknownType(q.to_owned())),
            1 => Ok(hits[0].id.clone()),
            _ => {
                let ids: Vec<String> = hits.iter().map(|h| h.id.to_string()).collect();
                Err(OntologyQueryError::AmbiguousType(format!(
                    "{q} → {}",
                    ids.join(", ")
                )))
            }
        }
    }

    /// Full type explorer report.
    pub fn type_report(&self, query: &str) -> Result<OntologyTypeReport, OntologyQueryError> {
        let id = self.resolve_selector(query)?;
        let summary = self
            .type_summary(&id)
            .ok_or_else(|| OntologyQueryError::UnknownType(query.to_owned()))?;
        let snap = self.snapshot();
        let refs = snap.reverse_references();
        let usages = refs
            .type_usages
            .get(id.as_str())
            .into_iter()
            .flatten()
            .map(|(owner, member)| OntologyReferenceHit {
                owner: owner.clone(),
                member: member.clone(),
            })
            .collect();
        let predicates: Vec<String> = snap
            .predicates()
            .filter(|p| {
                p.relation.source_type.as_str() == id.as_str()
                    || p.relation.target_type.as_str() == id.as_str()
            })
            .map(|p| p.id.to_string())
            .collect();
        let unallocated = snap.unallocated().iter().any(|u| u.unallocated == id)
            || id.local_name().starts_with("Unallocated");
        Ok(OntologyTypeReport {
            id: id.to_string(),
            direct_parents: summary.supertypes.iter().map(ToString::to_string).collect(),
            supertypes: self
                .supertypes(&id)
                .into_iter()
                .map(|t| t.to_string())
                .collect(),
            subtypes: self
                .subtypes(&id)
                .into_iter()
                .map(|t| t.to_string())
                .collect(),
            properties: self
                .properties(&id)
                .into_iter()
                .map(|p| p.id.to_string())
                .collect(),
            predicates,
            domains: summary.domains.iter().map(ToString::to_string).collect(),
            references: usages,
            evolution: snap.evolution_guidance(&id),
            unallocated,
            source_file: summary.provenance.source_file.clone(),
            source_package: summary.provenance.package.to_string(),
            source_revision: summary
                .source_revision
                .clone()
                .or_else(|| summary.provenance.commit.clone()),
            summary,
        })
    }

    /// Search report.
    pub fn search_report(&self, query: &str) -> OntologySearchReport {
        OntologySearchReport {
            query: query.to_owned(),
            types: self.find_types(query),
        }
    }

    /// Why-type report.
    pub fn why_report(&self, query: &str) -> Result<WhyType, OntologyQueryError> {
        let id = self.resolve_selector(query)?;
        self.why_type(&id)
            .ok_or_else(|| OntologyQueryError::UnknownType(query.to_owned()))
    }

    /// Reverse references for one type.
    pub fn references_report(
        &self,
        query: &str,
    ) -> Result<(TypeId, ReverseReferences, Vec<OntologyReferenceHit>), OntologyQueryError> {
        let id = self.resolve_selector(query)?;
        let all = self.references();
        let hits = all
            .type_usages
            .get(id.as_str())
            .into_iter()
            .flatten()
            .map(|(owner, member)| OntologyReferenceHit {
                owner: owner.clone(),
                member: member.clone(),
            })
            .collect();
        Ok((id, all, hits))
    }

    /// Impact report for a type.
    pub fn impact_report(&self, query: &str) -> Result<OntologyImpactReport, OntologyQueryError> {
        let id = self.resolve_selector(query)?;
        Ok(OntologyImpactReport {
            origin: id.to_string(),
            impact: self.impact(&id),
        })
    }

    /// Compatibility explanation (assignability).
    pub fn compatibility_report(
        &self,
        from: &str,
        to: &str,
    ) -> Result<WhyCompatible, OntologyQueryError> {
        let a = self.resolve_selector(from)?;
        let b = self.resolve_selector(to)?;
        Ok(self.why_compatible(&a, &b))
    }

    /// Diff two named ontology snapshots loaded on this SDK instance.
    ///
    /// Selectors are opaque strings (`package@version` or fingerprint). Loading
    /// Git history is an SDK concern — products only pass the selector.
    pub fn diff_report(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<OntologyDiffReport, OntologyQueryError> {
        let old = self.snapshot_for_selector(from)?;
        let new = self.snapshot_for_selector(to)?;
        Ok(OntologyDiffReport {
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
            diff: os20_ontology_core::ontology_diff(old, new),
        })
    }

    /// Migration analysis between named snapshots. Never rewrites IDs.
    pub fn migration_report(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<os20_ontology_core::MigrationReport, OntologyQueryError> {
        let old = self.snapshot_for_selector(from)?;
        let new = self.snapshot_for_selector(to)?;
        Ok(os20_ontology_core::migration_analysis(old, new))
    }

    fn snapshot_for_selector(
        &self,
        sel: Option<&str>,
    ) -> Result<&OntologySnapshot, OntologyQueryError> {
        let Some(raw) = sel.map(str::trim).filter(|s| !s.is_empty()) else {
            return Ok(self.snapshot());
        };
        if raw == self.snapshot().semantic_fingerprint().as_str()
            || raw == self.snapshot().source_fingerprint().as_str()
        {
            return Ok(self.snapshot());
        }
        if let Some(snap) = self.named_snapshot(raw) {
            return Ok(snap);
        }
        for (name, snap) in self.named_snapshots() {
            if name == raw {
                return Ok(snap);
            }
            if snap.semantic_fingerprint().as_str() == raw {
                return Ok(snap);
            }
            if snap.packages().any(|p| {
                format!("{}@{}", p.ontology_id, p.ontology_version) == raw
                    || p.ontology_id.as_str() == raw
            }) {
                return Ok(snap);
            }
        }
        if self.snapshot().packages().any(|p| {
            format!("{}@{}", p.ontology_id, p.ontology_version) == raw
                || p.ontology_id.as_str() == raw
        }) {
            return Ok(self.snapshot());
        }
        Err(OntologyQueryError::UnknownSnapshot(raw.to_owned()))
    }
}

fn source_kind_token(snap: &OntologySnapshot) -> &'static str {
    match snap.source_kind() {
        os20_ontology_core::OntologySourceKind::Bootstrap => "bootstrap",
        os20_ontology_core::OntologySourceKind::PackageBacked => "packageBacked",
    }
}
