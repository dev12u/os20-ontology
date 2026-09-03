//! OS20 SDK facade. Product callers use this crate, not `os20-ontology-core`.

#![forbid(unsafe_code)]

pub use os20_ontology_core::*;

mod ontology_facade;
mod reports;
mod sdk;

pub use ontology_facade::OntologyHandle;
pub use reports::{
    JsonEnvelope, OntologyDiffReport, OntologyImpactReport, OntologyPackageReport,
    OntologyQueryError, OntologyReferenceHit, OntologySearchReport, OntologyStatusReport,
    OntologyTypeReport, REPORT_SCHEMA, to_json_report,
};
pub use sdk::Os20Sdk;
