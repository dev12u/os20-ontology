//! `Os20Sdk` instance. Ontology context is per-instance, never process-global.

use std::collections::BTreeMap;

use os20_ontology_core::{
    MemoryOntology, OntologyCompileRequest, OntologyCompileResult, OntologyCompiler,
    OntologyRuntime, OntologySnapshot,
};

use crate::OntologyHandle;

/// SDK entry. Two instances may load different ontology versions in one process.
#[derive(Clone, Debug)]
pub struct Os20Sdk {
    ontology: OntologyRuntime,
    named: BTreeMap<String, OntologySnapshot>,
}

impl Os20Sdk {
    fn from_runtime(ontology: OntologyRuntime) -> Self {
        Self {
            ontology,
            named: BTreeMap::new(),
        }
    }

    /// Bootstrap via [`MemoryOntology::core()`] (emergency / existing behavior).
    pub fn bootstrap() -> Self {
        Self::from_runtime(OntologyRuntime::bootstrap())
    }

    /// Package-backed production ontology from a compile result.
    pub fn from_compile(result: OntologyCompileResult) -> Self {
        Self::from_runtime(OntologyRuntime::new(result.snapshot))
    }

    /// Compile bound SourceGraph facts / locked Git sources.
    pub fn compile_ontology(request: OntologyCompileRequest) -> OntologyCompileResult {
        OntologyCompiler::compile(request)
    }

    /// Attach an explicit ontology snapshot (package-backed or test fixtures).
    pub fn with_ontology_snapshot(snapshot: OntologySnapshot) -> Self {
        Self::from_runtime(OntologyRuntime::new(snapshot))
    }

    /// From a memory ontology.
    pub fn with_memory_ontology(ontology: MemoryOntology) -> Self {
        Self::with_ontology_snapshot(ontology.into_snapshot())
    }

    /// Register an additional named snapshot for version comparison (SDK-owned).
    ///
    /// Products pass the name as `--from` / `--to`; they do not load Git.
    #[must_use]
    pub fn with_named_ontology_snapshot(
        mut self,
        name: impl Into<String>,
        snapshot: OntologySnapshot,
    ) -> Self {
        self.named.insert(name.into(), snapshot);
        self
    }

    /// Ontology facade.
    pub fn ontology(&self) -> OntologyHandle<'_> {
        OntologyHandle::new(&self.ontology, &self.named)
    }
}
