//! Git tree / cache source reader. No working-tree checkout required.
//!
//! Locked offline reads cached blobs only (`FetchPolicy::Never`).

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use thiserror::Error;

/// Whether Git remotes / registry may be contacted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchPolicy {
    /// Locked offline: cache only.
    Never,
    /// Online update may fetch.
    Allow,
}

/// How ontology package versions are selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OntologyResolveMode {
    /// May consult the existing registry.
    Update,
    /// Exact `os20.lock` pins; no version updates.
    Locked,
    /// Exact pins from Git cache; zero network.
    LockedOffline,
}

/// Source-tree errors.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SourceTreeError {
    #[error("offline source unavailable: {0}")]
    OfflineMissing(String),
    #[error("malformed source: {0}")]
    Malformed(String),
}

/// Read ontology sources from an immutable Git tree or cache.
pub trait OntologySourceTree: Send + Sync {
    /// Registry HTTP calls observed (must stay 0 when locked offline).
    fn registry_calls(&self) -> u32 {
        0
    }

    /// Git fetch operations observed (must stay 0 when `FetchPolicy::Never`).
    fn git_fetches(&self) -> u32 {
        0
    }

    /// Read a blob. `commit` is the lock pin; `path` is package-root-relative
    /// or tree-absolute as stored in the cache key.
    fn read_blob(
        &self,
        commit: Option<&str>,
        path: &str,
        fetch: FetchPolicy,
    ) -> Result<Option<Vec<u8>>, SourceTreeError>;

    /// List files under a package root (deterministic order).
    fn list_files(
        &self,
        commit: Option<&str>,
        package_root: &str,
        fetch: FetchPolicy,
    ) -> Result<Vec<String>, SourceTreeError>;
}

/// In-memory Git object cache (bare source; no checkout).
#[derive(Debug, Default)]
pub struct GitObjectCache {
    /// `(commit, path) → bytes`. Empty commit key means workspace uncommitted.
    blobs: Mutex<BTreeMap<(String, String), Vec<u8>>>,
    fetches: AtomicU32,
    registry: AtomicU32,
}

impl GitObjectCache {
    /// Empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a cached blob (does not count as a fetch).
    pub fn insert(&self, commit: &str, path: &str, bytes: Vec<u8>) {
        self.blobs
            .lock()
            .expect("cache mutex")
            .insert((commit.to_owned(), path.to_owned()), bytes);
    }

    /// Record a registry call (tests / online update).
    pub fn note_registry_call(&self) {
        self.registry.fetch_add(1, Ordering::SeqCst);
    }

    /// Registry HTTP calls.
    pub fn registry_call_count(&self) -> u32 {
        self.registry.load(Ordering::SeqCst)
    }

    /// Git fetch count.
    pub fn git_fetch_count(&self) -> u32 {
        self.fetches.load(Ordering::SeqCst)
    }
}

impl OntologySourceTree for GitObjectCache {
    fn registry_calls(&self) -> u32 {
        self.registry.load(Ordering::SeqCst)
    }

    fn git_fetches(&self) -> u32 {
        self.fetches.load(Ordering::SeqCst)
    }

    fn read_blob(
        &self,
        commit: Option<&str>,
        path: &str,
        fetch: FetchPolicy,
    ) -> Result<Option<Vec<u8>>, SourceTreeError> {
        let key = (commit.unwrap_or("").to_owned(), path.to_owned());
        let map = self.blobs.lock().expect("cache mutex");
        if let Some(b) = map.get(&key) {
            return Ok(Some(b.clone()));
        }
        if fetch == FetchPolicy::Never {
            return Err(SourceTreeError::OfflineMissing(path.to_owned()));
        }
        drop(map);
        self.fetches.fetch_add(1, Ordering::SeqCst);
        Err(SourceTreeError::OfflineMissing(format!(
            "not in cache: {path}"
        )))
    }

    fn list_files(
        &self,
        commit: Option<&str>,
        package_root: &str,
        fetch: FetchPolicy,
    ) -> Result<Vec<String>, SourceTreeError> {
        let c = commit.unwrap_or("");
        let map = self.blobs.lock().expect("cache mutex");
        let mut files: Vec<String> = map
            .keys()
            .filter(|(cc, p)| cc == c && (package_root == "." || p.starts_with(package_root)))
            .map(|(_, p)| p.clone())
            .collect();
        if files.is_empty() && fetch == FetchPolicy::Never {
            return Err(SourceTreeError::OfflineMissing(package_root.to_owned()));
        }
        files.sort();
        Ok(files)
    }
}

/// Existing registry used only for version discovery (no ontology endpoint).
pub trait OntologyRegistry: Send + Sync {
    /// Resolve a package version range via the frozen registry API.
    fn resolve_version(&self, package: &str, range: &str) -> Result<String, SourceTreeError>;
    /// How many times the registry was contacted.
    fn calls(&self) -> u32;
}

/// Test registry.
#[derive(Debug, Default)]
pub struct RecordingRegistry {
    versions: Mutex<BTreeMap<String, String>>,
    calls: AtomicU32,
}

impl RecordingRegistry {
    /// Empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pin a latest version for Update mode.
    pub fn set_latest(&self, package: &str, version: &str) {
        self.versions
            .lock()
            .expect("registry mutex")
            .insert(package.to_owned(), version.to_owned());
    }

    /// Registry call count.
    pub fn call_count(&self) -> u32 {
        self.calls.load(Ordering::SeqCst)
    }
}

impl OntologyRegistry for RecordingRegistry {
    fn resolve_version(&self, package: &str, _range: &str) -> Result<String, SourceTreeError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.versions
            .lock()
            .expect("registry mutex")
            .get(package)
            .cloned()
            .ok_or_else(|| SourceTreeError::OfflineMissing(package.to_owned()))
    }

    fn calls(&self) -> u32 {
        self.calls.load(Ordering::SeqCst)
    }
}
