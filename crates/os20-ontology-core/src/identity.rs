//! Typed, named ontology identity. Never a database row id.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from parsing public ontology identities.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum IdentityError {
    #[error("empty identity")]
    Empty,
    #[error("identity must not be a database row id")]
    DatabaseIdForbidden,
    #[error("invalid package id `{0}` (expected `@scope/name`)")]
    Package(String),
    #[error("invalid element id `{0}` (expected `@scope/name#Local`)")]
    Element(String),
}

/// OS20 package identity (`@os20/core`). Version is not part of this id.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PackageId(String);

impl PackageId {
    /// Parse `@scope/name`.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, IdentityError> {
        let raw = raw.as_ref().trim();
        reject_db_id(raw)?;
        if !is_package_id(raw) {
            return Err(IdentityError::Package(raw.to_owned()));
        }
        Ok(Self(raw.to_owned()))
    }

    /// Stable `@os20/core` ontology package.
    pub fn os20_core() -> Self {
        Self::new("@os20/core").expect("catalog package")
    }

    /// `@os20/physics` extension package.
    pub fn os20_physics() -> Self {
        Self::new("@os20/physics").expect("catalog package")
    }

    /// `@os20/electrical` extension package.
    pub fn os20_electrical() -> Self {
        Self::new("@os20/electrical").expect("catalog package")
    }

    /// `@os20/mechanical` extension package.
    pub fn os20_mechanical() -> Self {
        Self::new("@os20/mechanical").expect("catalog package")
    }

    /// `@os20/behaviour` extension package.
    pub fn os20_behaviour() -> Self {
        Self::new("@os20/behaviour").expect("catalog package")
    }

    /// `@os20/engineering` v1 core vocabulary.
    pub fn os20_engineering() -> Self {
        Self::new("@os20/engineering").expect("catalog package")
    }

    /// `@os20/physical` geometry / materials / thermal / electrical structure.
    pub fn os20_physical() -> Self {
        Self::new("@os20/physical").expect("catalog package")
    }

    /// `@os20/software` embedded/software vocabulary.
    pub fn os20_software() -> Self {
        Self::new("@os20/software").expect("catalog package")
    }

    /// `@os20/assurance` requirements / test / safety classifications.
    pub fn os20_assurance() -> Self {
        Self::new("@os20/assurance").expect("catalog package")
    }

    /// `@os20/analysis` simulation and engineering analysis.
    pub fn os20_analysis() -> Self {
        Self::new("@os20/analysis").expect("catalog package")
    }

    /// `@os20/lifecycle` manufacturing, artifact roles, tools.
    pub fn os20_lifecycle() -> Self {
        Self::new("@os20/lifecycle").expect("catalog package")
    }

    /// Example model package (not an ontology role).
    pub fn os20_powertrain() -> Self {
        Self::new("@os20/powertrain").expect("catalog package")
    }

    /// Vendored KerML scalar library package.
    pub fn omg_kerml() -> Self {
        Self::new("@omg/kerml").expect("catalog package")
    }

    /// Borrow the canonical string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for PackageId {
    type Err = IdentityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Durable named element identity (`@os20/core#Thing`).
///
/// Rename of the local name changes this identity unless a future explicit
/// stable semantic id is introduced (see ADR 0005).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ElementId(String);

impl ElementId {
    /// Parse `@package#Local`.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, IdentityError> {
        let raw = raw.as_ref().trim();
        reject_db_id(raw)?;
        if !is_element_id(raw) {
            return Err(IdentityError::Element(raw.to_owned()));
        }
        Ok(Self(raw.to_owned()))
    }

    /// Construct from validated package + local name.
    pub fn qualified(package: &PackageId, local: &str) -> Result<Self, IdentityError> {
        if !is_local_name(local) {
            return Err(IdentityError::Element(format!("{package}#{local}")));
        }
        Ok(Self(format!("{package}#{local}")))
    }

    /// Catalog helper for known-good names.
    pub fn catalog(package: &PackageId, local: &str) -> Self {
        Self::qualified(package, local).expect("catalog element")
    }

    /// Borrow the canonical string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Package prefix.
    pub fn package(&self) -> Result<PackageId, IdentityError> {
        let pkg = self.0.split('#').next().unwrap_or("");
        PackageId::new(pkg)
    }

    /// Local name after `#`.
    pub fn local_name(&self) -> &str {
        self.0.split_once('#').map(|(_, n)| n).unwrap_or("")
    }
}

impl fmt::Display for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ElementId {
    type Err = IdentityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Typed classifier / datatype identity. Same string rules as [`ElementId`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypeId(ElementId);

impl TypeId {
    /// Parse a type element id.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, IdentityError> {
        Ok(Self(ElementId::new(raw)?))
    }

    /// Wrap an existing element id.
    pub fn from_element(id: ElementId) -> Self {
        Self(id)
    }

    /// `@os20/core#Thing`.
    pub fn thing() -> Self {
        Self::from_element(ElementId::catalog(&PackageId::os20_core(), "Thing"))
    }

    /// Borrow the underlying element identity.
    pub fn as_element(&self) -> &ElementId {
        &self.0
    }

    /// Canonical string.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Declaring package.
    pub fn package(&self) -> Result<PackageId, IdentityError> {
        self.0.package()
    }

    /// Local classifier name.
    pub fn local_name(&self) -> &str {
        self.0.local_name()
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<TypeId> for ElementId {
    fn from(value: TypeId) -> Self {
        value.0
    }
}

macro_rules! typed_element_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(ElementId);

        impl $name {
            /// Parse from `@package#Local`.
            pub fn new(raw: impl AsRef<str>) -> Result<Self, IdentityError> {
                Ok(Self(ElementId::new(raw)?))
            }

            /// Wrap an element id.
            pub fn from_element(id: ElementId) -> Self {
                Self(id)
            }

            /// Catalog constructor.
            pub fn catalog(package: &PackageId, local: &str) -> Self {
                Self(ElementId::catalog(package, local))
            }

            /// Underlying element id.
            pub fn as_element(&self) -> &ElementId {
                &self.0
            }

            /// Canonical string.
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<$name> for ElementId {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

typed_element_id!(
    OntologyId,
    "Ontology *semantic* package identity (not a release)."
);
typed_element_id!(PredicateId, "Stable predicate identity.");
typed_element_id!(PropertyId, "Stable feature/property identity.");
typed_element_id!(UnitId, "Stable unit identity (units track consumes this).");
typed_element_id!(
    SemanticDomainId,
    "Stable semantic domain identity (ontology-defined, versioned via its package release)."
);

/// Quantity type identity (physical quantity classifier).
pub type QuantityTypeId = TypeId;

fn reject_db_id(raw: &str) -> Result<(), IdentityError> {
    if raw.is_empty() {
        return Err(IdentityError::Empty);
    }
    let lower = raw.to_ascii_lowercase();
    if lower == "rowid" || lower.starts_with("rowid:") || raw.chars().all(|c| c.is_ascii_digit()) {
        return Err(IdentityError::DatabaseIdForbidden);
    }
    Ok(())
}

fn is_package_id(raw: &str) -> bool {
    let Some(rest) = raw.strip_prefix('@') else {
        return false;
    };
    let mut parts = rest.split('/');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(a), Some(b), None) if is_segment(a) && is_segment(b)
    )
}

fn is_element_id(raw: &str) -> bool {
    let Some((pkg, local)) = raw.split_once('#') else {
        return false;
    };
    is_package_id(pkg) && is_local_name(local)
}

fn is_segment(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn is_local_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_numeric_row_ids() {
        assert!(matches!(
            ElementId::new("42"),
            Err(IdentityError::DatabaseIdForbidden)
        ));
    }

    #[test]
    fn parses_named_identity() {
        let id = ElementId::new("@os20/core#Thing").unwrap();
        assert_eq!(id.local_name(), "Thing");
        assert_eq!(id.package().unwrap().as_str(), "@os20/core");
    }
}
