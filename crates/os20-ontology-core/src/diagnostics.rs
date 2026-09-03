//! Structured ontology diagnostics. Never panic on missing ontology.
//!
//! Ontology compiler codes use `OS20-E4xxx` / `OS20-W4xxx` and do not collide
//! with package (`E2xxx`) or language (`E3xxx`) series.

use serde::{Deserialize, Serialize};

use crate::identity::{ElementId, PackageId, TypeId};

/// Diagnostic code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticCode {
    /// Duplicate ontology element id (`OS20-E4001`).
    DuplicateOntologyId,
    /// Attempt to redefine a protected bootstrap id (`OS20-E4002`).
    ProtectedRedefinition,
    /// Type name did not bind (`OS20-E4003`).
    UnresolvedType,
    /// Package is not `PackageRole::Ontology` (`OS20-E4004`).
    IncompatibleOntologyRole,
    /// Invalid semantic-domain declaration (`OS20-E4005`).
    InvalidDomainDeclaration,
    /// Evolution mapping target missing (`OS20-E4006`).
    EvolutionTargetMissing,
    /// Required ontology package absent (`OS20-E4007`).
    MissingOntology,
    /// Locked/offline Git blob missing (`OS20-E4008`).
    MissingSourceOffline,
    /// Specialize cycle (`OS20-E4009`).
    SpecializationCycle,
    /// Package-import cycle among ontology packages (`OS20-E4010`).
    OntologyDependencyCycle,
    /// Malformed bound SourceGraph (`OS20-E4011`).
    MalformedOntologySource,
    /// Feature redefinition type is not a covariant specialization (`OS20-E4012`).
    IncompatibleOverride,
    /// Value not assignable to required type (`OS20-E4013`).
    IncompatibleAssignment,
    /// Quantity dimension mismatch (`OS20-E4014`).
    QuantityDimensionMismatch,
    /// Relation endpoint not allowed by the predicate (`OS20-E4015`).
    IllegalRelationEndpoint,
    /// Comment-only / advisory (`OS20-W4001`).
    CommentOnlyChange,
}

impl DiagnosticCode {
    /// Frozen diagnostic token.
    pub fn token(self) -> &'static str {
        match self {
            Self::DuplicateOntologyId => "OS20-E4001",
            Self::ProtectedRedefinition => "OS20-E4002",
            Self::UnresolvedType => "OS20-E4003",
            Self::IncompatibleOntologyRole => "OS20-E4004",
            Self::InvalidDomainDeclaration => "OS20-E4005",
            Self::EvolutionTargetMissing => "OS20-E4006",
            Self::MissingOntology => "OS20-E4007",
            Self::MissingSourceOffline => "OS20-E4008",
            Self::SpecializationCycle => "OS20-E4009",
            Self::OntologyDependencyCycle => "OS20-E4010",
            Self::MalformedOntologySource => "OS20-E4011",
            Self::IncompatibleOverride => "OS20-E4012",
            Self::IncompatibleAssignment => "OS20-E4013",
            Self::QuantityDimensionMismatch => "OS20-E4014",
            Self::IllegalRelationEndpoint => "OS20-E4015",
            Self::CommentOnlyChange => "OS20-W4001",
        }
    }

    /// Error vs warning.
    pub fn is_error(self) -> bool {
        !matches!(self, Self::CommentOnlyChange)
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.token())
    }
}

/// One diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyDiagnostic {
    /// Code.
    pub code: DiagnosticCode,
    /// Machine token (`OS20-E4xxx`).
    pub token: String,
    /// Message.
    pub message: String,
    /// Related package.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<PackageId>,
    /// Related element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<ElementId>,
    /// Cycle participants, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cycle: Vec<TypeId>,
    /// Related source locations (definition, offender, inherited contributor).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedLocation>,
}

/// A location attached to a type-error diagnostic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedLocation {
    /// Role of this location.
    pub role: String,
    /// Element.
    pub element: ElementId,
    /// Optional package-relative file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
}

impl OntologyDiagnostic {
    /// Construct with token filled from [`DiagnosticCode::token`].
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            token: code.token().to_owned(),
            code,
            message: message.into(),
            package: None,
            element: None,
            cycle: Vec::new(),
            related: Vec::new(),
        }
    }

    /// Missing ontology package.
    pub fn missing_ontology(package: PackageId) -> Self {
        Self::new(
            DiagnosticCode::MissingOntology,
            format!("required ontology package `{package}` is absent"),
        )
        .with_package(package)
    }

    /// Unresolved type (not coerced to Any).
    pub fn unresolved_type(written: &str) -> Self {
        Self::new(
            DiagnosticCode::UnresolvedType,
            format!("unresolved type `{written}`"),
        )
    }

    /// Specialization cycle (existing diagnostic engine shape).
    pub fn cycle(cycle: Vec<TypeId>) -> Self {
        let rendered = cycle
            .iter()
            .map(|t| t.as_str())
            .collect::<Vec<_>>()
            .join(" → ");
        let mut d = Self::new(
            DiagnosticCode::SpecializationCycle,
            format!("specialization cycle: {rendered}"),
        );
        d.cycle = cycle;
        d
    }

    /// Attach package.
    pub fn with_package(mut self, package: PackageId) -> Self {
        self.package = Some(package);
        self
    }

    /// Attach element.
    pub fn with_element(mut self, element: ElementId) -> Self {
        self.element = Some(element);
        self
    }

    /// Attach related locations.
    pub fn with_related(mut self, related: Vec<RelatedLocation>) -> Self {
        self.related = related;
        self
    }
}
