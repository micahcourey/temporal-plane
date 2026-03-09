//! Product-level maintenance and cleanup request contracts.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{BranchName, Checkpoint, RetentionPolicy, VersionNumber};

/// Describes the storage-level clone strategy used for an exported store.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CloneKind {
    /// Clone manifests and lineage cheaply while sharing existing data files.
    Shallow,
    /// Copy all dataset files for a fully isolated clone.
    Deep,
}

/// Describes a completed store clone operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloneInfo {
    destination: PathBuf,
    version_count: u64,
    kind: CloneKind,
}

impl CloneInfo {
    /// Creates clone metadata.
    #[must_use]
    pub fn new(destination: PathBuf, version_count: u64, kind: CloneKind) -> Self {
        Self {
            destination,
            version_count,
            kind,
        }
    }

    /// Returns the clone destination path.
    #[must_use]
    pub fn destination(&self) -> &Path {
        &self.destination
    }

    /// Returns the number of visible versions carried into the clone.
    #[must_use]
    pub const fn version_count(&self) -> u64 {
        self.version_count
    }

    /// Returns the clone strategy that was used.
    #[must_use]
    pub const fn kind(&self) -> CloneKind {
        self.kind
    }
}

/// A request to stage an import onto an isolated branch.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImportStageRequest {
    source_path: PathBuf,
    branch_name: Option<BranchName>,
}

impl ImportStageRequest {
    /// Creates an import staging request.
    #[must_use]
    pub fn new(source_path: impl Into<PathBuf>) -> Self {
        Self {
            source_path: source_path.into(),
            branch_name: None,
        }
    }

    /// Returns the source store path to import from.
    #[must_use]
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    /// Returns the optional destination staging branch override.
    #[must_use]
    pub const fn branch_name(&self) -> Option<&BranchName> {
        self.branch_name.as_ref()
    }

    /// Overrides the staging branch name.
    #[must_use]
    pub fn with_branch_name(mut self, branch_name: BranchName) -> Self {
        self.branch_name = Some(branch_name);
        self
    }
}

/// A result describing an isolated staged import.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImportStageResult {
    branch_name: BranchName,
    staged_records: u64,
    ready_to_merge: bool,
}

impl ImportStageResult {
    /// Creates an import staging result.
    #[must_use]
    pub fn new(branch_name: BranchName, staged_records: u64, ready_to_merge: bool) -> Self {
        Self {
            branch_name,
            staged_records,
            ready_to_merge,
        }
    }

    /// Returns the staging branch name.
    #[must_use]
    pub const fn branch_name(&self) -> &BranchName {
        &self.branch_name
    }

    /// Returns the number of records staged onto the branch.
    #[must_use]
    pub const fn staged_records(&self) -> u64 {
        self.staged_records
    }

    /// Returns `true` when the staged branch is ready for follow-up review.
    #[must_use]
    pub const fn ready_to_merge(&self) -> bool {
        self.ready_to_merge
    }
}

/// A request to run explicit store maintenance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OptimizeRequest {
    retention_policy: RetentionPolicy,
    prune_old_versions: bool,
}

impl OptimizeRequest {
    /// Creates a maintenance request using the provided retention policy.
    #[must_use]
    pub fn new(retention_policy: RetentionPolicy) -> Self {
        Self {
            retention_policy,
            prune_old_versions: false,
        }
    }

    /// Creates a maintenance request using conservative defaults.
    #[must_use]
    pub fn conservative() -> Self {
        Self::new(RetentionPolicy::conservative())
    }

    /// Returns the retention policy to apply.
    #[must_use]
    pub const fn retention_policy(&self) -> &RetentionPolicy {
        &self.retention_policy
    }

    /// Returns `true` when old versions should be pruned.
    #[must_use]
    pub const fn prune_old_versions(&self) -> bool {
        self.prune_old_versions
    }

    /// Enables or disables old-version pruning.
    #[must_use]
    pub fn with_prune_old_versions(mut self, prune_old_versions: bool) -> Self {
        self.prune_old_versions = prune_old_versions;
        self
    }
}

/// A product-level result returned by an optimize or cleanup flow.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OptimizeResult {
    previous_version: VersionNumber,
    current_version: VersionNumber,
    pre_optimize_checkpoint: Option<Checkpoint>,
    compacted: bool,
    pruned_versions: u64,
    bytes_removed: u64,
}

impl OptimizeResult {
    /// Creates a maintenance result.
    #[must_use]
    pub const fn new(
        previous_version: VersionNumber,
        current_version: VersionNumber,
        pre_optimize_checkpoint: Option<Checkpoint>,
        compacted: bool,
        pruned_versions: u64,
        bytes_removed: u64,
    ) -> Self {
        Self {
            previous_version,
            current_version,
            pre_optimize_checkpoint,
            compacted,
            pruned_versions,
            bytes_removed,
        }
    }

    /// Returns the version that was current before maintenance started.
    #[must_use]
    pub const fn previous_version(&self) -> VersionNumber {
        self.previous_version
    }

    /// Returns the version that is current after maintenance completed.
    #[must_use]
    pub const fn current_version(&self) -> VersionNumber {
        self.current_version
    }

    /// Returns the checkpoint created before optimization, if any.
    #[must_use]
    pub const fn pre_optimize_checkpoint(&self) -> Option<&Checkpoint> {
        self.pre_optimize_checkpoint.as_ref()
    }

    /// Returns whether compaction was attempted.
    #[must_use]
    pub const fn compacted(&self) -> bool {
        self.compacted
    }

    /// Returns the number of old versions pruned.
    #[must_use]
    pub const fn pruned_versions(&self) -> u64 {
        self.pruned_versions
    }

    /// Returns the number of bytes removed during pruning.
    #[must_use]
    pub const fn bytes_removed(&self) -> u64 {
        self.bytes_removed
    }
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use crate::{BranchName, CheckpointName, RecordedAt};

    use super::*;

    #[test]
    fn import_stage_request_preserves_branch_override() {
        let request = ImportStageRequest::new("/tmp/source-store")
            .with_branch_name(BranchName::try_from("import-stage").expect("valid branch"));

        assert_eq!(request.source_path(), Path::new("/tmp/source-store"));
        assert_eq!(
            request.branch_name().map(BranchName::as_str),
            Some("import-stage")
        );
    }

    #[test]
    fn clone_info_tracks_destination_and_kind() {
        let info = CloneInfo::new(PathBuf::from("/tmp/exported-store"), 4, CloneKind::Deep);

        assert_eq!(info.destination(), Path::new("/tmp/exported-store"));
        assert_eq!(info.version_count(), 4);
        assert_eq!(info.kind(), CloneKind::Deep);
    }

    #[test]
    fn optimize_request_defaults_to_non_pruning() {
        let request = OptimizeRequest::conservative();

        assert!(!request.prune_old_versions());
        assert_eq!(request.retention_policy(), &RetentionPolicy::conservative());
    }

    #[test]
    fn optimize_result_tracks_prune_stats() {
        let checkpoint = Checkpoint::new_at(
            CheckpointName::try_from("pre-optimize-v4").expect("valid checkpoint"),
            VersionNumber::new(4),
            RecordedAt::new(UNIX_EPOCH),
            Some("before compaction".to_string()),
        );
        let result = OptimizeResult::new(
            VersionNumber::new(4),
            VersionNumber::new(5),
            Some(checkpoint.clone()),
            true,
            2,
            4096,
        );

        assert_eq!(result.previous_version(), VersionNumber::new(4));
        assert_eq!(result.current_version(), VersionNumber::new(5));
        assert_eq!(result.pre_optimize_checkpoint(), Some(&checkpoint));
        assert_eq!(result.pruned_versions(), 2);
        assert_eq!(result.bytes_removed(), 4096);
        assert!(result.compacted());
    }
}
